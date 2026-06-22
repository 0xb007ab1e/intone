//! `oxeye-core` — platform-agnostic core of the **oxeye** screen reader.
//!
//! This crate holds everything that does **not** depend on a specific OS accessibility
//! API: the [`settings`] model, the user-defined [`exclusions`] engine, [`redaction`] of
//! sensitive content, and the [`untrusted`] trust-boundary wrapper for data read from the
//! accessibility tree. Platform back-ends (`oxeye-linux`, and later Windows/macOS) depend
//! on this crate and feed it data.
//!
//! # Security posture
//! A screen reader can observe everything on screen (passwords, banking, private messages)
//! and, by capturing keys, acts as a keylogger by function. Accordingly this crate:
//! - forbids `unsafe` code,
//! - treats all accessibility-tree text as [`untrusted::Untrusted`] until validated,
//! - never logs raw read content (see [`redaction`]),
//! - performs no network I/O and emits no telemetry.

pub mod announcement;
pub mod braille;
pub mod error;
pub mod exclusions;
pub mod navigation;
pub mod redaction;
pub mod settings;
pub mod untrusted;

pub use error::{Error, Result};
pub use exclusions::{Action, Context, ExclusionEngine, ExclusionRule};
pub use settings::{Settings, Speech, Verbosity};
pub use untrusted::Untrusted;
