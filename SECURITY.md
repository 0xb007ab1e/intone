# Security Policy

oxeye is a screen reader: it can observe everything on screen and captures keyboard input.
We treat it as a high-privilege, high-trust component and take security reports seriously.

## Reporting a vulnerability

Please report privately — do **not** open a public issue for a security problem.

- Preferred: GitHub private vulnerability reporting ("Report a vulnerability" on the
  repository's **Security** tab), or
- Email: `<security-contact-to-be-configured>`

We aim to acknowledge within **72 hours** and to coordinate disclosure: fix under embargo,
then publish an advisory and a patched release with credit.

## Scope (what we care about most)

- Any leakage of screen content or keystrokes — via logs, telemetry, IPC, temp files, or
  crash dumps.
- Any network egress that is not explicitly opted in by the user.
- Sandbox escapes in the add-on / scripting layer.
- Memory-safety issues (the core uses `#![forbid(unsafe_code)]`; report any `unsafe`
  introduced via dependencies).

## Our commitments

- **No telemetry and no network by default.** Any network feature is opt-in and documented.
- Read content is **redacted from logs by default** (`oxeye-core::redaction`,
  `oxeye-core::untrusted`).
- Dependencies are **license- and vulnerability-gated in CI** (`cargo-deny`, `cargo-audit`).
- Builds are reproducible; a pinned toolchain and an SBOM accompany releases.

See [`docs/security/threat-model.md`](docs/security/threat-model.md) for the STRIDE analysis.
