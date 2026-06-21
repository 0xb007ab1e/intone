//! Announcement composition: decides *what the screen reader actually says* for a focused
//! element, given the user's [`Verbosity`] preference and any matching exclusion [`Action`].
//!
//! This is the platform-agnostic **functional core** of announcement policy — pure and
//! deterministic, so it is unit-tested without any accessibility back-end. Platform crates
//! read an element from the accessibility tree, build a [`Context`], and call [`compose`].

use crate::exclusions::{Action, Context};
use crate::settings::Verbosity;

/// What to speak for an element, and how.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Announcement {
    /// The text to speak.
    pub text: String,
    /// Whether to interrupt in-progress speech. `false` for a de-prioritised
    /// ([`Action::LowerPriority`]) announcement, so it does not cut off current speech.
    pub interrupt: bool,
}

/// Maximum length of a summarised name before it is truncated (for [`Action::Summarize`]).
const SUMMARY_MAX_CHARS: usize = 40;

/// Compose the announcement for `ctx` under `verbosity`, honoring an optional exclusion
/// `action`.
///
/// Returns `None` when the element must not be announced at all ([`Action::Suppress`]).
/// [`Action::Summarize`] forces the shortened form regardless of verbosity;
/// [`Action::LowerPriority`] keeps the verbosity-appropriate text but marks it non-interrupting.
#[must_use]
pub fn compose(
    ctx: &Context<'_>,
    verbosity: Verbosity,
    action: Option<Action>,
) -> Option<Announcement> {
    let text = match action {
        Some(Action::Suppress) => return None,
        Some(Action::Summarize) => summary(ctx),
        _ => match verbosity {
            Verbosity::Low => concise(ctx),
            Verbosity::Medium => standard(ctx),
            Verbosity::High => detailed(ctx),
        },
    };
    let interrupt = action != Some(Action::LowerPriority);
    Some(Announcement { text, interrupt })
}

/// `Low`: the essential label only — the name, or the role when there is no name.
fn concise(ctx: &Context<'_>) -> String {
    if ctx.name.is_empty() {
        ctx.role.to_owned()
    } else {
        ctx.name.to_owned()
    }
}

/// `Medium` (default): "name, role", or just the role when unnamed.
fn standard(ctx: &Context<'_>) -> String {
    if ctx.name.is_empty() {
        ctx.role.to_owned()
    } else {
        format!("{}, {}", ctx.name, ctx.role)
    }
}

/// `High`: the standard form plus the owning application, when known.
fn detailed(ctx: &Context<'_>) -> String {
    let base = standard(ctx);
    if ctx.app.is_empty() {
        base
    } else {
        format!("{base}, {}", ctx.app)
    }
}

/// A shortened announcement: the first line of the name, length-capped, plus the role.
fn summary(ctx: &Context<'_>) -> String {
    let first_line = ctx.name.lines().next().unwrap_or(ctx.name).trim();
    let mut short: String = first_line.chars().take(SUMMARY_MAX_CHARS).collect();
    if first_line.chars().count() > SUMMARY_MAX_CHARS {
        short.push('…');
    }
    if short.is_empty() {
        ctx.role.to_owned()
    } else {
        format!("{short}, {}", ctx.role)
    }
}

#[cfg(test)]
mod tests {
    use super::{compose, Announcement};
    use crate::exclusions::{Action, Context};
    use crate::settings::Verbosity;

    fn ctx<'a>(name: &'a str, role: &'a str, app: &'a str) -> Context<'a> {
        Context { name, role, app }
    }

    fn text(c: &Context<'_>, v: Verbosity) -> String {
        compose(c, v, None).unwrap().text
    }

    #[test]
    fn verbosity_controls_detail() {
        let c = ctx("OK", "push button", "installer");
        assert_eq!(text(&c, Verbosity::Low), "OK");
        assert_eq!(text(&c, Verbosity::Medium), "OK, push button");
        assert_eq!(text(&c, Verbosity::High), "OK, push button, installer");
    }

    #[test]
    fn unnamed_element_falls_back_to_role_at_every_level() {
        let c = ctx("", "panel", "installer");
        assert_eq!(text(&c, Verbosity::Low), "panel");
        assert_eq!(text(&c, Verbosity::Medium), "panel");
        // High still appends the app even when unnamed.
        assert_eq!(text(&c, Verbosity::High), "panel, installer");
    }

    #[test]
    fn suppress_yields_nothing() {
        let c = ctx("secret", "label", "bank");
        assert_eq!(compose(&c, Verbosity::High, Some(Action::Suppress)), None);
    }

    #[test]
    fn summarize_overrides_verbosity_and_truncates() {
        let long = "x".repeat(100);
        let c = ctx(&long, "banner", "web");
        let ann = compose(&c, Verbosity::High, Some(Action::Summarize)).unwrap();
        assert!(ann.text.ends_with(", banner"));
        assert!(ann.text.contains('…'));
        assert!(ann.interrupt);
    }

    #[test]
    fn lower_priority_keeps_text_but_does_not_interrupt() {
        let c = ctx("Loading", "statusbar", "ide");
        let ann = compose(&c, Verbosity::Medium, Some(Action::LowerPriority)).unwrap();
        assert_eq!(
            ann,
            Announcement {
                text: "Loading, statusbar".to_owned(),
                interrupt: false,
            }
        );
    }
}
