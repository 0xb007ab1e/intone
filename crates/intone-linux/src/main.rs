//! intone-linux — binary entry point.
//!
//! The screen-reader logic lives in the `intone_linux` library (`lib.rs`) so it can be
//! reused and tested independently; this binary only sets up the async runtime and runs it.

/// Set up the Tokio runtime and run the Linux back-end.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    intone_linux::run().await
}
