# intone — Threat Model (STRIDE)

_Status: living document. Revisit on any change to a trust boundary._
_Method: STRIDE over the data-flow diagram (per `workflow-threat-model`)._

## System & data flow

intone reads the platform accessibility tree, optionally captures keys, applies the
exclusions/verbosity policy, and emits speech (and later braille). On the maintainer's
target (Parrot 7, KDE Plasma 6 / Wayland) the platform mechanisms are:

```
[apps] --AT-SPI2--> [intone-linux back-end] --> [intone-core policy] --> [speech-dispatcher] --> audio
                          ^ KWin a11y KeyboardMonitor (key capture)
[settings/exclusions] <--0600--> local disk           [add-ons] -- sandboxed --> core API
```

All inter-process channels are **local, same-user, same-session** (session D-Bus, Unix
sockets), mediated by the kernel and filesystem permissions — **not** the network.

## Trust boundaries

1. Application UI → intone — accessibility-tree data is **untrusted input** (`Untrusted<T>`).
2. Keyboard → intone — the capture path; intone is a keylogger by function.
3. intone ↔ disk — settings/exclusions at rest.
4. intone ↔ add-ons/scripts — extensibility is the **largest attack surface**.
5. intone ↔ network — **only if** an optional, opt-in networked feature is added later.

## Data classification

- **Restricted:** screen content and keystrokes (may contain passwords, PII, financial data).
- **Internal:** user settings and exclusion rules (reveal app usage).

## On "encryption in transit" / mTLS

There is **no network transport in the core**, so TLS/mTLS do not apply to the local D-Bus /
Unix-socket IPC (secured by kernel + filesystem permissions). TLS 1.3 and, where mutual
authentication is warranted, **mTLS are mandated for any future networked feature** (remote
access, add-on downloads, optional cloud voices) — boundary 5. Applying them to localhost IPC
would be security theatre, not a control.

## STRIDE

| Threat | Where | Mitigation |
|--------|-------|-----------|
| **S**poofing | add-ons impersonating core; future network peers | capability-scoped add-on API; mTLS + identity for any future network peer |
| **T**ampering | settings file; add-ons | `0600`/`0700` perms; signed add-ons (planned); validate config, fail closed |
| **R**epudiation | misbehaving add-on | audit add-on actions (redacted) in local logs |
| **I**nfo disclosure | **primary risk** — leaking screen/keys via logs, telemetry, temp files, IPC | redaction by default; `Untrusted<T>`; **no telemetry/network by default**; no secrets in logs; zeroize transient secrets |
| **D**enial of service | malformed a11y data; ReDoS via user regex | bound/validate inputs; linear-time `regex` engine; never block the speech path on tree queries |
| **E**levation of privilege | add-on/script escaping its sandbox | least-privilege sandbox; explicit typed capabilities; human approval for sensitive actions |

## Key controls (implemented or scaffolded)

- `#![forbid(unsafe_code)]` in the core (memory safety).
- `redaction` + `Untrusted<T>` (info-disclosure containment).
- Network off by default; verifiable via reproducible OSS builds.
- Supply chain: pinned toolchain, `cargo-deny` (licenses + advisories), `cargo-audit`, SBOM.
- Secure config perms (`0600`/`0700`); at-rest encryption path via the OS keyring
  (KWallet / libsecret) using a vetted AEAD — to be implemented when sensitive data is
  actually stored at rest.

## Abuse cases → tests

- A rule with a malformed regex must be **rejected** (fails closed) — covered by
  `exclusions::tests::invalid_regex_fails_closed`.
- `Untrusted<T>` must never reveal contents in `Debug` — covered by
  `untrusted::tests::debug_never_reveals_contents`.
- Defaults must be offline (no network) — covered by
  `settings::tests::defaults_are_private_and_offline`.
