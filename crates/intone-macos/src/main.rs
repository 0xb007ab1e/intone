//! `intone-macos` binary — the macOS (Accessibility / AXAPI) screen-reader back-end.

fn main() -> anyhow::Result<()> {
    intone_macos::run()
}
