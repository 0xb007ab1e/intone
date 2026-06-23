//! `oxeye-cli` — the testable core of the `oxeye` configuration command.
//!
//! This library holds the **disk-free** rule-mutation and formatting logic so it can be unit
//! tested without touching the filesystem (the imperative shell — loading/saving settings and
//! printing — lives in `main.rs`). It depends only on [`oxeye_core`], keeping the core itself
//! free of any CLI dependency.

use anyhow::{ensure, Context, Result};
use oxeye_core::{Action, ExclusionEngine, ExclusionRule, Settings, Verbosity};

/// Add `rule` to `settings`, validating it first. Fails **closed**.
///
/// # Errors
/// Returns an error if the rule has no matchers (it would match every announcement) or if its
/// `name_regex` does not compile — in either case `settings` is left unchanged.
pub fn add_rule(settings: &mut Settings, rule: ExclusionRule) -> Result<()> {
    ensure!(
        rule.app.is_some() || rule.role.is_some() || rule.name_regex.is_some(),
        "refusing to add a rule with no matchers — it would match every announcement; \
         set at least one of --app / --role / --name-regex"
    );
    // Validate the regex by compiling the rule; refuse to persist a malformed rule.
    ExclusionEngine::compile(std::slice::from_ref(&rule)).context("invalid --name-regex")?;
    settings.exclusions.push(rule);
    Ok(())
}

/// Remove the rule numbered `position` (1-based, as printed by [`format_list`]).
///
/// # Errors
/// Returns an error if `position` is out of range; `settings` is left unchanged.
pub fn remove_rule(settings: &mut Settings, position: usize) -> Result<ExclusionRule> {
    let count = settings.exclusions.len();
    ensure!(
        position >= 1 && position <= count,
        "no rule #{position}; there are {count} rule(s) — see `oxeye exclusions list`"
    );
    Ok(settings.exclusions.remove(position - 1))
}

/// Render the configured rules as a numbered, human-readable list.
#[must_use]
pub fn format_list(settings: &Settings) -> String {
    if settings.exclusions.is_empty() {
        return "no exclusion rules configured".to_owned();
    }
    let mut lines = Vec::with_capacity(settings.exclusions.len());
    for (i, rule) in settings.exclusions.iter().enumerate() {
        let mut matchers = Vec::new();
        if let Some(app) = &rule.app {
            matchers.push(format!("app={app}"));
        }
        if let Some(role) = &rule.role {
            matchers.push(format!("role={role}"));
        }
        if let Some(re) = &rule.name_regex {
            matchers.push(format!("name~={re}"));
        }
        let matchers = if matchers.is_empty() {
            "(any)".to_owned()
        } else {
            matchers.join(" ")
        };
        lines.push(format!(
            "{}. [{}] {matchers}",
            i + 1,
            action_label(rule.action)
        ));
    }
    lines.join("\n")
}

/// Stable, lowercase label for an [`Action`] — used in listings and matching the `--action`
/// value names accepted on the command line.
#[must_use]
pub fn action_label(action: Action) -> &'static str {
    match action {
        Action::Suppress => "suppress",
        Action::Summarize => "summarize",
        Action::LowerPriority => "lower-priority",
    }
}

/// Stable, lowercase label for a [`Verbosity`] level (matches the CLI value names).
#[must_use]
pub fn verbosity_label(verbosity: Verbosity) -> &'static str {
    match verbosity {
        Verbosity::Low => "low",
        Verbosity::Medium => "medium",
        Verbosity::High => "high",
    }
}

/// Validate a 0–100 speech level (rate / pitch / volume). Fails **closed** above 100 rather
/// than silently clamping, so a typo is reported instead of quietly applied.
///
/// # Errors
/// Returns an error if `value` exceeds 100.
pub fn checked_level(value: u8) -> Result<u8> {
    ensure!(value <= 100, "level must be 0–100 (got {value})");
    Ok(value)
}

/// Interpret a CLI value for an optional speech setting (voice / language / output module):
/// the literal `default` clears it (revert to the engine default); anything else sets it.
#[must_use]
pub fn optional_setting(value: &str) -> Option<String> {
    match value {
        "default" => None,
        other => Some(other.to_owned()),
    }
}

/// A short, human-readable summary of the current configuration.
#[must_use]
pub fn format_config(settings: &Settings) -> String {
    let network = if settings.allow_network {
        "allowed"
    } else {
        "off"
    };
    let braille = if settings.braille { "on" } else { "off" };
    let speech = &settings.speech;
    let or_default = |opt: &Option<String>| opt.clone().unwrap_or_else(|| "default".to_owned());
    format!(
        "verbosity: {}\n\
         network: {network}\n\
         braille: {braille}\n\
         speech: rate {}, pitch {}, volume {}\n\
         voice: {}\n\
         language: {}\n\
         output module: {}\n\
         exclusion rules: {}",
        verbosity_label(settings.verbosity),
        speech.rate,
        speech.pitch,
        speech.volume,
        or_default(&speech.voice),
        or_default(&speech.language),
        or_default(&speech.output_module),
        settings.exclusions.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::{action_label, add_rule, format_list, remove_rule};
    use oxeye_core::{Action, ExclusionRule, Settings};

    fn rule(app: Option<&str>, name: Option<&str>, action: Action) -> ExclusionRule {
        ExclusionRule {
            app: app.map(str::to_owned),
            role: None,
            name_regex: name.map(str::to_owned),
            action,
        }
    }

    #[test]
    fn add_then_remove_roundtrips() {
        let mut s = Settings::default();
        add_rule(&mut s, rule(Some("noisyapp"), None, Action::Suppress)).unwrap();
        assert_eq!(s.exclusions.len(), 1);
        let removed = remove_rule(&mut s, 1).unwrap();
        assert_eq!(removed.action, Action::Suppress);
        assert!(s.exclusions.is_empty());
    }

    #[test]
    fn add_rejects_empty_matcher() {
        let mut s = Settings::default();
        assert!(add_rule(&mut s, rule(None, None, Action::Suppress)).is_err());
        assert!(
            s.exclusions.is_empty(),
            "a rejected rule must not be stored"
        );
    }

    #[test]
    fn add_rejects_invalid_regex() {
        let mut s = Settings::default();
        assert!(add_rule(&mut s, rule(None, Some("(unclosed"), Action::Summarize)).is_err());
        assert!(s.exclusions.is_empty());
    }

    #[test]
    fn remove_out_of_range_is_error() {
        let mut s = Settings::default();
        add_rule(&mut s, rule(Some("a"), None, Action::Suppress)).unwrap();
        assert!(
            remove_rule(&mut s, 0).is_err(),
            "1-based: index 0 is invalid"
        );
        assert!(remove_rule(&mut s, 2).is_err(), "out of range");
        assert_eq!(s.exclusions.len(), 1, "failed removals leave rules intact");
    }

    #[test]
    fn format_list_numbers_rules_and_handles_empty() {
        let mut s = Settings::default();
        assert!(format_list(&s).contains("no exclusion rules"));
        add_rule(
            &mut s,
            rule(Some("web"), Some("(?i)cookie"), Action::Summarize),
        )
        .unwrap();
        let listed = format_list(&s);
        assert!(listed.contains("1."), "rules are numbered");
        assert!(listed.contains("summarize"));
        assert!(listed.contains("app=web"));
        assert!(listed.contains("name~=(?i)cookie"));
    }

    #[test]
    fn action_labels_are_stable() {
        assert_eq!(action_label(Action::Suppress), "suppress");
        assert_eq!(action_label(Action::Summarize), "summarize");
        assert_eq!(action_label(Action::LowerPriority), "lower-priority");
    }

    #[test]
    fn verbosity_labels_are_stable() {
        use oxeye_core::Verbosity;
        assert_eq!(super::verbosity_label(Verbosity::Low), "low");
        assert_eq!(super::verbosity_label(Verbosity::Medium), "medium");
        assert_eq!(super::verbosity_label(Verbosity::High), "high");
    }

    #[test]
    fn config_summary_reports_verbosity_and_counts() {
        let mut s = Settings::default();
        add_rule(&mut s, rule(Some("a"), None, Action::Suppress)).unwrap();
        let out = super::format_config(&s);
        assert!(out.contains("verbosity: medium"), "default verbosity");
        assert!(out.contains("network: off"), "network off by default");
        assert!(out.contains("braille: off"), "braille off by default");
        assert!(out.contains("exclusion rules: 1"));
    }

    #[test]
    fn config_summary_reports_speech_defaults() {
        let out = super::format_config(&Settings::default());
        assert!(
            out.contains("speech: rate 50, pitch 50, volume 100"),
            "speech defaults shown"
        );
        assert!(out.contains("voice: default"), "voice unset shows default");
        assert!(out.contains("output module: default"));
    }

    #[test]
    fn config_summary_reports_a_set_voice() {
        let mut s = Settings::default();
        s.speech.voice = Some("Alan".to_owned());
        s.speech.rate = 70;
        let out = super::format_config(&s);
        assert!(out.contains("voice: Alan"));
        assert!(out.contains("rate 70"));
    }

    #[test]
    fn checked_level_accepts_0_to_100_and_rejects_above() {
        assert_eq!(super::checked_level(0).unwrap(), 0);
        assert_eq!(super::checked_level(100).unwrap(), 100);
        assert!(super::checked_level(101).is_err(), "fails closed above 100");
    }

    #[test]
    fn optional_setting_treats_default_as_clear() {
        assert_eq!(super::optional_setting("default"), None);
        assert_eq!(super::optional_setting("Alan"), Some("Alan".to_owned()));
    }
}
