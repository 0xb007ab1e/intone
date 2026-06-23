//! Redaction of sensitive content for logs and telemetry.
//!
//! A screen reader reads text that frequently includes passwords, tokens, and personal
//! data. Raw content must never reach logs, traces, or any sink. These helpers produce
//! non-revealing stand-ins and decide what may be spoken for protected fields.

/// Replace text content with a non-revealing placeholder safe to record in logs.
///
/// Only the character count is retained, never the content.
#[must_use]
pub fn redact_for_log(text: &str) -> String {
    format!("<redacted: {} chars>", text.chars().count())
}

/// The text a screen reader should speak for a value.
///
/// Content from a protected (password) field is never spoken verbatim.
#[must_use]
pub fn speakable(text: &str, protected: bool) -> String {
    if protected {
        "protected entry".to_owned()
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{redact_for_log, speakable};

    #[test]
    fn redaction_keeps_only_length() {
        assert_eq!(redact_for_log("hunter2"), "<redacted: 7 chars>");
    }

    #[test]
    fn redaction_counts_unicode_scalars() {
        assert_eq!(redact_for_log("héllo"), "<redacted: 5 chars>");
    }

    #[test]
    fn protected_fields_are_never_spoken_verbatim() {
        assert_eq!(speakable("s3cret", true), "protected entry");
        assert_eq!(speakable("Inbox", false), "Inbox");
    }
}
