//! A trust-boundary wrapper for data that originates outside intone — primarily text and
//! metadata read from the platform accessibility tree, which is produced by arbitrary
//! third-party applications and must be treated as hostile input.

use std::fmt;

/// Wraps a value that came from an untrusted source (e.g. the accessibility tree).
///
/// The wrapped value cannot be read without an explicit call to [`Untrusted::expose`] or
/// [`Untrusted::into_inner`], which makes every crossing of the trust boundary visible at
/// the call site. The [`fmt::Debug`] implementation never reveals the contents, so an
/// `Untrusted<T>` is safe to include in log/telemetry context (see also
/// [`crate::redaction`]).
#[derive(Clone)]
pub struct Untrusted<T>(T);

impl<T> Untrusted<T> {
    /// Wrap a value coming from an untrusted source.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Borrow the inner value, explicitly crossing the trust boundary.
    ///
    /// The caller is responsible for validating/encoding the value before use.
    #[must_use]
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Consume the wrapper and return the inner value, crossing the trust boundary.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Untrusted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Untrusted(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::Untrusted;

    #[test]
    fn debug_never_reveals_contents() {
        let secret = Untrusted::new("hunter2");
        assert_eq!(format!("{secret:?}"), "Untrusted(<redacted>)");
    }

    #[test]
    fn expose_and_into_inner_return_value() {
        let u = Untrusted::new(42);
        assert_eq!(*u.expose(), 42);
        assert_eq!(u.into_inner(), 42);
    }
}
