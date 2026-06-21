# Feasibility: An OSS, Cross-Platform, Privacy-Respecting Screen Reader

**Date:** 2026-06-21
**Status:** Feasibility inquiry (no code yet)
> **Update (2026-06-21):** Direction set to **Linux-first**, then Windows/macOS. The
> platform ordering in §7 is superseded by **`LINUX-FIRST-PLAN.md`**; the rest of this
> analysis (complaints, remediations, no-tracking, exclusions, risks) still applies.

**Question:** Is it feasible to build an open-source, cross-platform screen reader with
no tracking, user-defined exclusions, and remediation of the common complaints leveled at
existing screen readers?

---

## TL;DR — Verdict

**Feasible, but with a sharply scoped definition of "cross-platform."**

- A single screen-reader *core* (settings, speech routing, scripting, exclusions, command
  model, braille routing) is very feasible to share across OSes.
- The *platform layer* (how you read the UI and intercept keys) is **fundamentally
  different on each OS** and is where ~70% of the real work and risk lives. There is no
  shared accessibility API; you write three back-ends.
- "No tracking" is the **easiest** requirement to satisfy and a genuine differentiator —
  it's mostly a matter of discipline, not engineering.
- "User exclusions" is **straightforward** and a real, under-served need.
- "Fix all the common complaints" is the **dangerous** part of the goal. Many complaints
  (latency, web nav, app compatibility) are not bugs — they are the *core difficulty* of
  the entire product category. Treat them as the multi-year engineering mission, not a
  checklist item.

**Recommended framing:** Don't build "one screen reader for all platforms." Build a
**shared core + best-in-class Windows screen reader first** (where the OSS gap is real and
the API is the most tractable), with the architecture designed so macOS and Linux back-ends
can follow. A true simultaneous tri-platform v1 is a recipe for shipping nothing.

---

## 1. The hard part: there is no cross-platform accessibility API

A screen reader must (a) read the UI's accessibility tree, (b) receive events when it
changes, and (c) intercept/inject keyboard input globally. Each OS exposes this completely
differently:

| OS | Accessibility API | Notable property |
|----|-------------------|------------------|
| Windows | **UI Automation (UIA)** (+ legacy MSAA/IA2) | Can read a large chunk of the tree at once; richest tooling; best OSS opportunity |
| macOS | **NSAccessibility / AXUIElement (AXAPI)** | Batch reads possible; **requires Accessibility permission**; Apple tightly controls the input layer |
| Linux | **AT-SPI2** (over D-Bus) | Must query **each property individually over D-Bus** → inherently chattier/slower; Wayland adds gaps (e.g. GTK4 key-event delivery) |

Implications:

- You are writing **three back-ends**, not one app. The shared core is real and worth it,
  but budget for the platform layer being the dominant cost.
- **Performance characteristics differ per OS.** AT-SPI2's per-property D-Bus round-trips
  are a known source of Linux sluggishness (cf. Orca freezes) — your Linux back-end needs
  aggressive caching/batching from day one.
- **Wayland** is a moving target: legacy global key interception and some event paths don't
  work the way they did on X11. This is an active area (GNOME is reworking this in 2025–26)
  and a real risk for the Linux target.
- **The browser is its own accessibility tree.** Chromium/Firefox expose web content
  through the platform API but with their own quirks; web navigation is the single most
  complaint-heavy surface (see §2) and needs dedicated engineering on every platform.

**Key building block — AccessKit** (`github.com/AccessKit/accesskit`): a Rust project
providing a cross-platform abstraction that maps a single accessibility model onto UIA /
AXAPI / AT-SPI2. *Caveat:* AccessKit is designed for **apps to expose** accessibility, not
primarily for an AT to **consume** it — but its platform adapters, data model, and the
expertise of its authors (veterans of the AT world) make it the most relevant prior art and
possibly a dependency or design reference. This is the first thing to evaluate hands-on.

---

## 2. Common complaints (grounded) → remediation strategy

Complaints below are drawn from user communities and reporting on NVDA, JAWS, VoiceOver,
Narrator, and Orca (see Sources). Each is paired with a realistic remediation posture.

| # | Complaint (and where it bites) | Can a new OSS reader remediate it? | How |
|---|--------------------------------|-----------------------------------|-----|
| 1 | **Cost** — JAWS runs ~$90–$1,475/yr; unaffordable globally, blocks devs from testing | **Yes — by definition** | OSS + free is the whole premise. Biggest, most certain win. |
| 2 | **Web navigation breakage** — VoiceOver skipping text/blank pages after updates; NVDA slow on the web | **Partially; this is the core mission** | Treat web as a first-class engine; regression-test against real sites; ship fixes on *your* cadence, not the OS vendor's. |
| 3 | **Latency / responsiveness** — "too slow," lag under load | **Partially** | Async, non-blocking arch; cache the a11y tree; never block speech on tree queries. Honest: hard, never "solved." |
| 4 | **App/3rd-party compatibility** — silence in non-Microsoft apps (Narrator), Qt on Orca, unlabeled buttons | **Partially** | Per-app overlays/scripts (see #7); strong toolkit coverage (Qt, Electron, Java, GTK). Compatibility is endless maintenance. |
| 5 | **Stability** — NVDA stops talking; Orca random freezes; "buggy mess" sentiment for VoiceOver | **Yes, with discipline** | Crash isolation between core and platform back-ends; watchdog that auto-restarts speech; deterministic, tested core. |
| 6 | **Verbosity / poor defaults** — redundant announcements slow users down | **Yes** | Sensible concise defaults + granular per-context verbosity control. Cheap, high-impact UX win. |
| 7 | **Weak customization/scripting** — Narrator has none; JAWS scripting is proprietary/locked | **Yes — differentiator** | Embed a *safe, sandboxed* scripting layer (e.g. Lua/Python/WASM) with an open API. Community add-ons (NVDA's model proves demand). |
| 8 | **Braille support gaps** — uneven across readers; integration is hard | **Eventually** | Use **liblouis** (the de-facto OSS braille translation lib). Defer to a later phase; it's deep. |
| 9 | **TTS / voice quality** — robotic, unintelligible (Orca), or limited voices | **Partially** | Pluggable TTS: system voices + eSpeak NG (fast, low-latency) + optional neural voices (offline, opt-in). Don't bundle cloud TTS (tracking risk). |
| 10 | **Privacy / telemetry distrust** | **Yes — by definition** | See §3. |
| 11 | **No / clumsy exclusions** — can't tell the reader to ignore noisy regions, chatty apps, or specific controls | **Yes — requested feature** | See §4. |

**The honest line:** #1, #6, #7, #10, #11 are achievable wins a small team can deliver and
that meaningfully differentiate. #2, #3, #4, #8, #9 are *the perpetual hard core of the
category* — you don't "remediate" them once; you commit to out-engineering incumbents on
them over years. Promising to fix all of them in "this version" is the main way this project
would fail.

---

## 3. "No tracking" — the easiest requirement, and a real differentiator

This is mostly discipline, and cheap to guarantee:

- **No telemetry, no analytics, no phone-home** by default. No account, no cloud.
- **Local-only by default.** Any network feature (update check, optional neural voice
  download, add-on store) is **explicit opt-in**, documented, and individually toggleable.
- **No cloud TTS/OCR in the default build.** If offered, it's opt-in and clearly labeled as
  leaving the device. (Note: OCR/"screen recognition" is a *missing feature* users want —
  e.g. macOS has none natively — but the privacy-preserving answer is **on-device** OCR,
  e.g. Tesseract, not a cloud service.)
- **Privacy is verifiable because it's OSS** — reproducible builds + SBOM let users/distros
  confirm there's no exfiltration. This is a marketing and trust asset, not just a feature.
- Crash logs/diagnostics stay **local**; sharing is a manual, user-initiated export.

Aligns with the SSDLC mandates already in force here (no tracking, data minimization,
redaction, reproducible/signed releases).

---

## 4. User-defined exclusions — straightforward and under-served

A genuinely useful, tractable feature. Scope it as a rules engine:

- **Exclude by:** application/process, window, role/control type, ARIA/region, regex on
  accessible name, or coordinates/region of screen.
- **Actions:** suppress announcement, lower priority, summarize, or route differently
  (e.g. "never read this status bar," "mute this chatty app," "skip cookie banners").
- **Scope:** global, per-app, per-site (for the web engine), with an easy "exclude this
  thing" hotkey that captures the focused element and offers a rule.
- **Storage:** plain, human-readable, version-controllable config (no opaque binary). Shareable
  exclusion rule-sets become community content.

Risk: a poorly designed exclusion engine can hide important info or add latency. Keep rule
evaluation O(small) and in the core, not the hot speech path.

---

## 5. Proposed architecture & stack

```
        ┌─────────────────────────────────────────────────┐
        │                  Shared Core                     │
        │  command model · settings · exclusions engine ·  │
        │  verbosity/announcement policy · scripting host · │
        │  speech/braille routing · update (opt-in)        │
        └───────▲───────────────▲──────────────▲───────────┘
   platform back-ends (one each; the bulk of the work)
        ┌───────┴──────┐ ┌──────┴───────┐ ┌────┴─────────┐
        │  Windows     │ │   macOS      │ │   Linux      │
        │  UIA + key   │ │  AXAPI +     │ │  AT-SPI2 +   │
        │  hooks       │ │  CGEvent     │ │  D-Bus, X11/ │
        │              │ │  (perms!)    │ │  Wayland     │
        └──────────────┘ └──────────────┘ └──────────────┘
   pluggable outputs:  TTS (eSpeak NG / system / neural-offline)
                       Braille (liblouis + display drivers)
```

**Language/stack recommendation: Rust core.**
- Memory-safe (a screen reader is long-running, parses untrusted UI data, and is a
  high-trust component — matches the master ruleset's memory-safety preference).
- AccessKit and the relevant ecosystem are Rust; strong FFI to each OS API.
- Prior art exists (`rust-reader` and others), confirming viability.
- Alternative: C++ (NVDA's core is Python + C++; mature but heavier). Python-for-core (NVDA's
  model) eases community add-ons but costs latency — Rust core + sandboxed scripting layer
  gets both.

**Scripting/add-ons:** embed a sandboxed runtime (Lua, or WASM for language-agnostic
add-ons, or Python via an embedded interpreter à la NVDA). Open, documented API. This
directly remediates complaint #7 and bootstraps a community.

**TTS:** eSpeak NG as the always-available low-latency baseline; system voices via each OS;
optional offline neural voices (e.g. Piper) as opt-in downloads.

---

## 6. Honest risks & blockers

1. **Scope/ambition mismatch (highest risk).** "Cross-platform AND fixes every complaint AND
   v1" will sink it. Sequence ruthlessly.
2. **Platform layer cost.** Three back-ends, each deep. Wayland/Linux input + AT-SPI2 perf
   are the gnarliest.
3. **macOS friction.** Apple gates AT capabilities behind permissions and offers AT vendors
   far less latitude than Windows; a fully capable macOS back-end is the hardest of the three.
4. **The "compatibility tail" never ends.** Apps and the web break constantly; this is
   permanent maintenance, not a milestone. Needs a sustainable contributor base.
5. **Accuracy/safety bar is high.** Blind users *depend* on correctness; a screen reader that
   lies about UI state or goes silent is worse than annoying — it's harmful. Determinism,
   testing, and a self-healing speech watchdog are non-negotiable.
6. **Sustainability.** NVDA succeeds on donations + a nonprofit (NV Access) + a paid ecosystem
   around it. Plan governance/funding early or it stalls.
7. **Community trust & co-design.** This must be built *with* blind users from day one, not
   for them. Lived-experience testing is a hard requirement, not QA polish.

---

## 7. Recommended path (phased)

- **Phase 0 — Spike (weeks).** Hands-on evaluation of **AccessKit** as consumer vs. raw
  UIA. Prototype: on Windows, read the focused element + speak it via eSpeak NG, intercept
  one hotkey. Prove the core↔back-end seam. *Decision gate: build vs. contribute to an
  existing OSS reader (e.g. could some goals be met by improving NVDA add-ons / Orca?).*
- **Phase 1 — Windows MVP.** Shared core + Windows back-end. Focus, navigation, basic web
  reading, concise defaults, **exclusions engine**, **no-tracking** posture, scripting host
  skeleton. Ship something real to real users.
- **Phase 2 — Differentiators.** Open scripting/add-on API + add-on sharing; per-app overlays
  for compatibility; offline neural TTS option; OCR (on-device).
- **Phase 3 — Linux back-end (AT-SPI2).** Tackle perf (caching/batching) and Wayland head-on.
- **Phase 4 — macOS back-end (AXAPI).** Hardest; do last, informed by the prior two.
- **Phase 5 — Braille (liblouis)** + display drivers, across platforms.

A **"build vs. contribute" decision in Phase 0 is the single most important checkpoint** —
the world may be better served by you supercharging NVDA/Orca than by a fourth from-scratch
reader, *unless* the cross-platform + no-tracking + exclusions combination is the specific
gap you mean to fill (it plausibly is).

---

## 8. Bottom line

- **Technically feasible?** Yes. The pieces exist (AccessKit, eSpeak NG, liblouis,
  per-OS APIs, Rust ecosystem, NVDA as an existence proof).
- **Feasible as "one v1 that does everything on every OS"?** No. That framing fails.
- **Worth doing?** The *combination* — free/OSS + verifiably no-tracking + powerful
  exclusions + open scripting — is a real, unfilled niche. The cost/access and
  privacy/control wins are achievable by a small team; the web/latency/compatibility "core
  hard problems" are a long-haul commitment.
- **Right next step:** a 2–4 week Windows spike to validate the architecture seam and make
  the build-vs-contribute decision before committing to the full platform matrix.

---

## Sources

- NVDA problems/performance — [nvda.groups.io](https://nvda.groups.io/g/nvda/topic/problems_with_nvda/29684701), [GitHub issue #8182](https://github.com/nvaccess/nvda/issues/8182)
- JAWS cost/affordability — [WebAIM: JAWS license not developer friendly](https://webaim.org/blog/jaws-license-not-developer-friendly/), [Double Tap: JAWS pricing](https://doubletaponair.com/breaking-down-jaws-pricing-ai-features-and-accessibility-options/), [Eric Eggert: Set JAWS free](https://yatil.net/blog/set-jaws-free)
- VoiceOver bugs/web nav — [AppleVis 2025 Report Card](https://www.applevis.com/blog/apple-vision-accessibility-2025-applevis-report-card), [AppleVis: state of VoiceOver for macOS](https://www.applevis.com/forum/macos-mac-apps/state-voiceover-macos-superior-suppressed), [AppleVis: 15.7 web nav issues](https://www.applevis.com/forum/macos-mac-apps/submitted-feedback-very-serious-web-navigation-issues-voiceover-mac-introduced)
- Narrator limitations — [BOIA: Why users avoid Narrator](https://www.boia.org/blog/why-do-screen-reader-users-avoid-microsoft-narrator), [AFB: Current State of Narrator](https://afb.org/aw/18/10/15273)
- Orca limitations — [Linux Mint Forums](https://forums.linuxmint.com/viewtopic.php?t=289019), [LWN: Enhancing screen-reader functionality in modern GNOME](https://lwn.net/Articles/1025127/), [GitHub: lxqt Qt nav issue](https://github.com/lxqt/lxqt/issues/2632)
- Accessibility APIs / cross-platform — [AccessKit](https://github.com/AccessKit/accesskit), [W3C Core-AAM 1.2](https://www.w3.org/TR/core-aam-1.2/), [crowecawcaw: cross-platform desktop automation via a11y APIs](https://crowecawcaw.github.io/general/2026/05/30/accessibility-for-computer-use.html), [Chromium accessibility overview](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md)
- Braille / verbosity — [Helen Keller Services: Between the Dots](https://www.helenkeller.org/between-the-dots-what-designers-miss-without-braille-users/)
- Prior OSS art — [rust-reader](https://github.com/Eh2406/rust-reader), [GitHub screen-reader topic](https://github.com/topics/screen-reader)
