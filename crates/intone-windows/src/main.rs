//! `intone-windows` binary — the Windows (UI Automation) screen-reader back-end.
//!
//! Sets up nothing platform-specific itself; it just calls [`intone_windows::run`], which on
//! Windows drives UI Automation and on other hosts reports that the back-end is Windows-only.

fn main() -> anyhow::Result<()> {
    intone_windows::run()
}
