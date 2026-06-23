//! Error types for `intone-core`.
//!
//! Expected failures are modelled explicitly and the crate fails closed: callers decide on
//! degraded behaviour, and nothing proceeds on ambiguous state.

use std::path::PathBuf;

use thiserror::Error;

/// Errors produced by `intone-core`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// No per-user configuration directory could be determined for this platform.
    #[error("could not determine a configuration directory for the current user")]
    NoConfigDir,

    /// A filesystem operation on the configuration failed.
    #[error("configuration I/O error at {path}")]
    Io {
        /// Path that was being accessed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The configuration file could not be parsed.
    #[error("failed to parse configuration")]
    Parse(#[from] toml::de::Error),

    /// The configuration could not be serialised.
    #[error("failed to serialise configuration")]
    Serialize(#[from] toml::ser::Error),

    /// An exclusion rule contained an invalid regular expression.
    #[error("invalid exclusion-rule regular expression")]
    InvalidRegex(#[from] regex::Error),
}

/// Convenience alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;
