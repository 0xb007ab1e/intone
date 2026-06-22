//! `oxeye-windows` — the Windows back-end of the **oxeye** screen reader.
//!
//! This crate is deliberately thin: it adapts Windows **UI Automation** (UIA) — reading the
//! focused element and (later) its events and speech — and hands the data to the reusable,
//! platform-agnostic policy in [`oxeye_core`] (announcement composition, exclusions, verbosity,
//! navigation, braille). The same core that drives `oxeye-linux` drives this; only the
//! accessibility-tree, event, and output adapters differ.
//!
//! UIA is reached through COM, which is an FFI boundary requiring `unsafe`; that is confined to
//! the [`uia`] module (see this crate's `unsafe_code = "allow"` lint override). `oxeye-core`
//! itself remains `unsafe`-free.

#[cfg(windows)]
mod uia;

/// Run the Windows screen-reader back-end.
///
/// On Windows this initializes UI Automation and reads focus; on other hosts it returns an
/// error so the workspace still builds and the binary fails cleanly.
///
/// # Errors
/// Propagates UI Automation / COM initialization failures (Windows), or a "Windows only" error
/// on other platforms.
#[cfg(windows)]
pub fn run() -> anyhow::Result<()> {
    uia::run()
}

/// Stub entry point on non-Windows hosts: the back-end requires the UI Automation APIs.
///
/// # Errors
/// Always returns an error indicating the current platform is unsupported.
#[cfg(not(windows))]
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!(
        "oxeye-windows requires the Windows UI Automation APIs; this host is {}",
        std::env::consts::OS
    )
}
