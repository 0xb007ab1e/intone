# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Pre-1.0, the public API may change
between minor versions.

## [0.1.0] — 2026-06-22

First public release: a free, open-source, **privacy-respecting**, **cross-platform** screen
reader built core-first in Rust. No telemetry; networking is off by default.

### Added

- **`oxeye-core`** — platform-agnostic policy, `unsafe`-free and I/O-free:
  - user-defined **exclusions** (suppress / summarize / lower-priority by app · role · name regex);
  - **announcement composition** scaled by **verbosity** (low / medium / high);
  - **structured-navigation** classification + document-order next/previous search;
  - **Grade-1 braille** translation (text → Unicode braille patterns);
  - `Untrusted<T>` trust boundary and log **redaction**; hardened (`0600`) settings storage.
- **`oxeye-linux`** — AT-SPI2 back-end (KDE Plasma / Wayland verified):
  - focus reading; element **states**, numeric **value**, and single-line text **content**;
  - **caret tracking**, **edit** (insert/delete) and **selection** announcements — password-gated;
  - **structured navigation**: `Ctrl+Alt+S` structure summary; `Ctrl+Alt+{H,B,L,F}` by type
    (`Shift` = previous);
  - **speech** via speech-dispatcher (SSIP) and **braille** rendering; global keys via KWin's
    accessibility `KeyboardMonitor`; `OXEYE_SPEECH=text` for headless/remote use.
- **`oxeye-windows`** — UI Automation back-end (compiled in CI against the real Windows SDK):
  - event-driven focus; **states** (checked/expanded/selected/disabled/required) and **value**;
  - **SAPI** speech; **by-type navigation** `Ctrl+Alt+{H,B,L,F}` via `RegisterHotKey`.
- **`oxeye-cli`** (`oxeye`) — manage configuration: exclusion rules and `config verbosity|braille`.
- Dual-licensed **MIT OR Apache-2.0**. Merge-blocking CI: format, clippy, tests, `cargo-audit`,
  `cargo-deny` (license + advisories), SBOM, and a Windows compile job.

### Known limitations

- The Windows back-end is **compile-verified** in CI but not yet runtime-tested on a real
  desktop. Braille **device** output (BrlAPI) is designed but not wired (see
  `docs/braille-transport.md`); macOS (AXAPI) is planned. Heading navigation on Linux/Windows and
  `has_popup` on Windows have documented edge cases. See open issues.

[0.1.0]: https://github.com/0xb007ab1e/oxeye/releases/tag/v0.1.0
