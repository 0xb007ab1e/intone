# Linux-First Strategy

**Date:** 2026-06-21
**Decision:** Build a workable **Linux** screen reader first, then expand to Windows/macOS.
**Companion doc:** see `FEASIBILITY.md` for the overall feasibility analysis.

---

## Why Linux-first is a smart entry point (right now especially)

- **The Linux a11y foundation is being rebuilt — and you can ride the wave.** The legacy
  **AT-SPI2** stack (D-Bus, *pull* model, per-property round-trips → the latency behind
  Orca's sluggishness, plus Wayland gaps) is being superseded by **Newton**: a
  Wayland-native, *push* model where apps publish their whole accessibility tree to the
  compositor → the AT. Newton is built on **AccessKit** and designed for sandboxed apps.
  Prototypes exist for AccessKit, Orca, and Mutter today.
- **Newton/AccessKit is cross-platform by design.** Because it's the same data model that
  maps onto Windows UIA and macOS AXAPI, **building your reader on the AccessKit consumer
  side makes the later Windows/macOS expansion far cheaper.** Linux-first is not a detour
  from your end goal — it's the cheapest on-ramp to it.
- **The OSS gap is real on Linux.** Orca is excellent but **GNOME-centric and Python-based**
  (a latency ceiling), and coverage is thin outside GNOME (KDE/Plasma, wlroots compositors
  like Sway, many Qt apps). No-tracking and rich user-exclusions aren't the focus anywhere.

---

## Decisions locked in (from the maintainer)

- **Build new, not improve-Orca.** Goal is to "cover the most area and user base in the end
  through incremental adoption" — that requires a clean, reusable core, which patching Orca
  (GNOME-tied, Python) would not provide. *Reusing Orca's LGPL heuristics where convenient
  stays on the table* (no licensing concern); the architecture is ours.
- **Stack:** Rust core on an **AccessKit/AT-SPI consumer abstraction**, so the same core
  later carries Windows (UIA) and macOS (AXAPI). Windows/Mac are explicitly a **future
  iteration**, not now.
- **Both X11 and Wayland** are in scope as first-class compatibility targets (not X11-only).
- **Primary design target = the maintainer's daily driver: stock Parrot OS.** *(Pending: is
  it Parrot 6 = MATE/X11/GTK, or Parrot 7 "Echo" = KDE Plasma 6/Wayland/Qt? — determines the
  MVP's toolkit + display server; see below.)*

## ✅ Verified on the actual target machine (2026-06-21)

Maintainer's box: **Parrot 7 "Echo", KDE Plasma 6.3.6, KWin Wayland**. Live probes confirm
every hard primitive a screen reader needs is available through **sanctioned, compositor-
blessed APIs** — no evdev hack, no root, no fighting Parrot's hardened Wayland:

| Primitive | Mechanism present on this box | Status |
|-----------|------------------------------|--------|
| **Read UI tree + events** | AT-SPI2 — `org.a11y.Bus` up, `at-spi2-registryd` running, `QT_ACCESSIBILITY=1`, atk-bridge loaded | ✅ working now |
| **Capture/grab global keys on Wayland** | **KWin owns `org.freedesktop.a11y.Manager` → `org.freedesktop.a11y.KeyboardMonitor`** at `/org/freedesktop/a11y/Manager`, methods: `WatchKeyboard`/`UnwatchKeyboard` (monitor), `SetKeyGrabs` (`aua(uu)`, grab specific combos), `GrabKeyboard`/`UngrabKeyboard` | ✅ present now |
| Alt. input paths | `InputCapture` + `RemoteDesktop` + `GlobalShortcuts` portals advertised; `kde.portal` backend; `libeis.so.1` installed; evdev as last resort (user not in `input` group) | ✅ fallbacks exist |
| **Speech output** | **none installed** — no speech-dispatcher / espeak-ng / piper | ❌ must `apt install` (trivial) |

**Conclusion:** the feared "Wayland can't capture keys" blocker **does not apply here** —
KDE solved it with `KeyboardMonitor`. Remaining work is engineering, not "is it possible."
The one missing piece (a speech engine) is a one-line install, not a design problem.

## ✅ Phase-0 spike — working (2026-06-21)

`crates/intone-linux` is a runnable spike proving the full seam on the target box (Parrot 7,
KDE Plasma 6 / Wayland): **AT-SPI2 focus events → `intone-core` exclusions policy →
speech-dispatcher**. It speaks the name + role of each element as focus moves — the core
screen-reader behaviour. Verified: compiles (MSRV-aware 1.85 graph; zbus 5.13.2), connects
to the live a11y bus, registers for `StateChanged`/`focused` events, and drives speech.

Run it in a graphical KDE session (needs the a11y bus + audio):

```text
cargo run -p intone-linux      # then Tab/Alt-Tab around and listen
```

Speech (increment #1, done): a **persistent SSIP connection** to speech-dispatcher with
**interrupt-on-focus** (cancels the prior utterance the instant focus moves) and rate control
from settings; **auto-spawns** the daemon if its socket is absent (the pure-Rust SSIP client,
unlike libspeechd, doesn't). Pure Rust over the local socket — GPL TTS engines stay in their
own process, keeping intone permissively licensed.

Hotkeys (increment #2, done): global keys via KWin's `org.freedesktop.a11y.KeyboardMonitor`
(the sanctioned Wayland key path) — **Control silences** speech, **Pause repeats** the last
announcement. AT-SPI focus events and key events are handled concurrently (`tokio::select!`).
Keys are *watched* (pass-through), not grabbed.

**Live-test findings (KWin 6.3.6, verified on the target):** (1) the SSIP daemon must be
polled for, not waited-on with a fixed delay (cold start is slow); (2) KWin authorises
`KeyboardMonitor` **only for the owner of the well-known name `org.gnome.Orca.KeyboardMonitor`**
(hardcoded in `a11ykeyboardmonitor.cpp`) — intone must claim that name on the same connection
before `WatchKeyboard`. Also set `org.a11y.Status.ScreenReaderEnabled`. (3) the `atspi` proxy's **default property
caching crashed Qt apps' a11y bridge** (a `GetAll` on build → SIGSEGV in the app) — fixed by
`cache_properties(No)` (lazy reads). TODO: reset a11y flags + release the name on exit (SIGINT).

**Headless/remote testing (done):** `INTONE_SPEECH=text` prints announcements (no audio, no
daemon) for remote dev over SSH/tmux; `scripts/test-readout.sh` launches test apps, captures
the readout, and tears everything down (self-cleaning, no idle leftovers). **Verified reading
real elements:** kdialog → `OK, button`. (kcalc opens focused on an unnamed panel → `, panel`;
expected — Wayland blocks synthetic Tab, so the harness can't advance focus within an app.)

Spike limitations (next increments): application name not yet resolved, so per-app exclusion
rules can't match on it; dedicated *consumed* shortcuts will use `SetKeyGrabs`; refactor the
back-end into a library + thin app binary before adding Windows/macOS.

## The Parrot OS wrinkle (target environment)

Parrot OS **7.0 "Echo"** (Dec 2025, Debian 13) changed the default desktop **from MATE on
X11 to KDE Plasma 6 on Wayland**. So the MVP target depends on the installed version:

| Parrot version | Desktop | Toolkit | Display server | Difficulty |
|----------------|---------|---------|----------------|-----------|
| 6.x and earlier | **MATE** | GTK3 | **X11** | **Easiest** — AT-SPI2 mature on X11; global key grab trivial |
| 7.x "Echo" | **KDE Plasma 6** | **Qt** | **Wayland** | **Hardest** — Qt a11y quirks + Wayland key-capture blocked by design |

**The security-distro irony:** Parrot promotes Wayland's **anti-keylogging** as a feature —
which is the *same* mechanism a screen reader needs to read keystrokes globally. On Parrot 7,
the OS is deliberately resisting what this app must do. Key capture there must go through
sanctioned channels (compositor a11y/input-method/virtual-keyboard protocols), not a grab.

**Toolkit-agnostic by default:** **AT-SPI2 normalizes both GTK and Qt**, so the tree-reading
layer is largely shared between MATE and KDE. The dominant variable is **X11 vs Wayland**
(the input/interception layer), not GTK vs Qt.

---

## The Linux-specific technical landscape (what you're actually integrating)

| Concern | Standard Linux mechanism | Notes / risk |
|---------|--------------------------|--------------|
| **Read UI tree + events** | **AT-SPI2** today; **Newton** (AccessKit-based) emerging | Design a consumer abstraction over AccessKit's model; consume AT-SPI2 now (incl. Newton's AT-SPI-emulating Python compat lib), adopt Newton natively as it lands. **Do not hard-couple to raw AT-SPI.** |
| **Speech output** | **speech-dispatcher** (the standard Linux speech layer) → eSpeak NG / others | Use speech-dispatcher rather than driving TTS engines directly; it's what every Linux AT targets. eSpeak NG = low-latency baseline; Piper = opt-in offline neural. |
| **Braille** | **BRLTTY** + **liblouis** (translation) | Defer to a later phase; deep but well-trodden OSS. |
| **Keyboard interception (the hard part on Wayland)** | X11: global grabs are easy. **Wayland: no global key grab by design** | Needs compositor cooperation: input-method / virtual-keyboard / a11y protocols, or a Newton-era input path. This is the **#1 Linux technical risk** — validate in the spike. |
| **OCR (missing-feature win)** | **Tesseract** on-device | Privacy-preserving alternative to cloud "screen recognition." Later phase. |

---

## Carried-over differentiators (from FEASIBILITY.md, all still apply)

- **Verifiable no-tracking** — no telemetry/cloud by default; reproducible builds prove it.
- **User-defined exclusions engine** — exclude by app/role/name-regex/region → suppress /
  summarize / mute; "exclude this thing" hotkey; human-readable, shareable rules.
- **Open, sandboxed scripting / add-ons** — the thing locked up in JAWS and absent in
  Narrator; NVDA proves the demand.
- **Concise, sane verbosity defaults** with granular per-context control.

---

## Phased roadmap (Linux-first)

- **Phase 0 — Spike (2–4 weeks).** On the maintainer's Parrot desktop: consume the a11y tree
  via **AT-SPI2** (toolkit-agnostic — works for MATE/GTK *and* KDE/Qt), speak the focused
  element through **speech-dispatcher + eSpeak NG**, and **prove keyboard interception on the
  target display server** (trivial on X11; the riskiest unknown on Wayland). Evaluate
  AccessKit consumer-side maturity vs. raw AT-SPI. **Decision gate: confirm display-server
  input path before committing to Phase 1.**
- **Phase 1 — Linux MVP.** Rust core + Linux back-end: focus tracking, caret/object nav,
  basic web reading (Firefox/Chromium a11y tree), **exclusions engine**, concise defaults,
  **no-tracking** posture, scripting host skeleton. Ship to real users on the maintainer's
  Parrot stack first.
- **Phase 2 — Differentiators + reach.** Add the *other* display server (X11↔Wayland) and
  desktop (MATE↔KDE↔GNOME↔wlroots/Sway) — incremental adoption toward "most area." Open
  scripting/add-on API + sharing; per-app fixes; offline neural TTS (Piper); on-device OCR
  (Tesseract).
- **Phase 3 — Braille** (BRLTTY + liblouis).
- **Phase 4 — Windows back-end (UIA)** — cheap*er* because the core already speaks AccessKit's
  model.
- **Phase 5 — macOS back-end (AXAPI)** — hardest; last.

---

## Honest risks specific to Linux-first

1. **Building on shifting ground (AT-SPI → Newton).** Mitigate by abstracting over
   AccessKit's model, not raw AT-SPI.
2. **Wayland keyboard interception — sharpened by the target.** The single biggest technical
   unknown, and worse on Parrot 7's hardened Wayland (anti-keylogging by design). Spike the
   actual target display server first. On X11 this risk largely evaporates.
3. **Smaller blind-Linux user base** = a smaller initial test/contributor community than
   Windows would give. Recruit co-designers early.
4. **Fragmentation tax** if you chase every desktop/display-server at once. Nail the
   maintainer's Parrot stack first, then expand one axis at a time (display server, then
   desktop) toward broad coverage.
5. **Sustainability/governance** — plan funding + lived-experience co-design from day one
   (Orca/NV Access show what it takes).

---

## Licensing & compliance automation (decided)

- **Commit the license files once** (`LICENSE-MIT` + `LICENSE-APACHE`); the Apache curl is a
  one-time bootstrap, **not** a build step. Fetching a project's own license fresh on every
  build is an anti-pattern (breaks reproducible builds, risks shipping a license-less
  artifact, adds a mutable network dependency — `std-supplychain`, `workflow-cicd`).
- **Automate dependency-license *compliance*, not license fetching**, in CI:
  - `cargo-deny check` — gate dependency licenses against the permissive allowlist
    (enforces the GPL-at-arm's-length strategy) + block vulnerable/yanked crates.
  - `cargo-about` — regenerate `THIRD-PARTY-NOTICES` (this *does* change as deps change).
  - `cargo-cyclonedx` — SBOM with per-component license data.
- Wired into `.github/workflows/ci.yml`; policy in `deny.toml`.

## Sources (Linux-specific)

- Newton / Wayland-native a11y — [GNOME a11y: Update on Newton](https://blogs.gnome.org/a11y/2024/06/18/update-on-newton-the-wayland-native-accessibility-project/), [at-spi2-core devel-docs: next-gen protocol](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/new-protocol.html), [LWN: Modernizing accessibility for desktop Linux](https://lwn.net/Articles/971541/), [LWN: Accessibility in Wayland](https://lwn.net/Articles/980811/)
- Wayland a11y research notes — [splondike/wayland-accessibility-notes](https://github.com/splondike/wayland-accessibility-notes)
- Orca — [GNOME/orca GitLab](https://gitlab.gnome.org/GNOME/orca), [orca.gnome.org](https://orca.gnome.org/), [LWN: Enhancing screen-reader functionality in modern GNOME](https://lwn.net/Articles/1025127/)
- AccessKit — [github.com/AccessKit/accesskit](https://github.com/AccessKit/accesskit)
