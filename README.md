# oxeye

> Working name (easily renamed). A free, open-source, **cross-platform**,
> **privacy-respecting** screen reader — built core-first so the same engine carries
> Linux → Windows → macOS.

**Status:** pre-spike (feasibility verified). See [`FEASIBILITY.md`](FEASIBILITY.md) and
[`LINUX-FIRST-PLAN.md`](LINUX-FIRST-PLAN.md).

## What makes it different

- **No tracking, ever.** No telemetry, no accounts, no cloud by default. Any network feature
  is explicit, individually toggleable opt-in. Verifiable because it's open source
  (reproducible builds).
- **User-defined exclusions.** Tell the reader to ignore noisy regions, chatty apps, or
  specific controls — by app / role / accessible-name regex / region. Human-readable,
  shareable rules.
- **Open, sandboxed extensibility.** A documented add-on/scripting API (the thing locked away
  in JAWS and absent in Narrator), so the community can extend behavior per app/site.
- **Concise, sane defaults** with granular verbosity control.
- **Cross-platform by construction.** A reusable Rust core on an AccessKit/AT-SPI model;
  Linux first (KDE/Wayland verified), Windows (UIA) and macOS (AXAPI) as later iterations.

## License

**Dual-licensed: [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), at your option.**

Chosen to be **as permissive as possible** and to **maximize reuse and extensibility across
every platform** — the standard for foundational cross-platform Rust projects (including
[AccessKit](https://github.com/AccessKit/accesskit), a likely dependency):

- **MIT** — maximally permissive, minimal obligations.
- **Apache-2.0** — adds an explicit **patent grant**, so companies and contributors can adopt
  and extend without patent risk.
- **"MIT OR Apache-2.0"** lets each downstream consumer pick whichever they prefer.

### Keeping the permissive promise (dependency-license strategy)

The Linux speech/braille stack is copyleft; it's kept **at arm's length** so no copyleft
reaches oxeye's code (license-compliance — see the project ruleset):

| Dependency | License | How we use it | Effect |
|------------|---------|---------------|--------|
| speech-dispatcher / eSpeak NG | GPL / LGPL / GPLv3 | **IPC** (separate process, SSIP socket) | none — not linked |
| liblouis (braille, later) | LGPL | **dynamic link** | LGPL permits this from a permissive app |
| AccessKit | MIT OR Apache-2.0 | linked | compatible |

Do **not** statically link or vendor GPL code into the core.

> **Setup note:** `LICENSE-APACHE` must contain the canonical Apache-2.0 text. Populate it
> authoritatively with:
> `curl -fsSL https://www.apache.org/licenses/LICENSE-2.0.txt -o LICENSE-APACHE`

## Architecture (intended)

```
oxeye-core   — reusable, platform-agnostic: command model, settings, exclusions engine,
               verbosity/announcement policy, scripting host, speech/braille routing
oxeye-linux  — AT-SPI2 tree reader + KWin a11y KeyboardMonitor input (Wayland verified);
               speech-dispatcher output
(later) oxeye-windows (UIA), oxeye-macos (AXAPI)
```

## Verified target environment

Parrot OS 7 "Echo", KDE Plasma 6 / KWin Wayland: AT-SPI2 tree access works; global key
capture available via KWin's `org.freedesktop.a11y.KeyboardMonitor`; speech engine needs
install (`speech-dispatcher` + `espeak-ng`). Details in `LINUX-FIRST-PLAN.md`.

## Running

```sh
cargo run -p oxeye-linux                      # speak (needs audio + speech-dispatcher)
OXEYE_SPEECH=text cargo run -p oxeye-linux    # print announcements (headless/remote dev)
```

Developing remotely and want to *hear* it? Either use `OXEYE_SPEECH=text`, or route the audio
to your machine over SSH/tailnet — see [`docs/remote-audio.md`](docs/remote-audio.md)
(`scripts/remote-audio.sh` automates it).
