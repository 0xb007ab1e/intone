//! `oxeye` — command-line configuration manager for the oxeye screen reader.
//!
//! Currently manages user-defined **exclusion rules** (the rules that tell the reader to
//! suppress, summarise, or de-prioritise announcements). The disk-free logic lives in the
//! `oxeye_cli` library; this binary is the imperative shell: parse args, load/save settings,
//! print results.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use oxeye_core::{Action, ExclusionRule, Settings};

/// Configure the oxeye screen reader.
#[derive(Parser)]
#[command(name = "oxeye", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage user-defined exclusion rules.
    Exclusions {
        #[command(subcommand)]
        command: ExclusionsCommand,
    },
}

#[derive(Subcommand)]
enum ExclusionsCommand {
    /// List configured exclusion rules.
    List,
    /// Add an exclusion rule (at least one matcher is required).
    Add {
        /// Match a specific application name.
        #[arg(long)]
        app: Option<String>,
        /// Match a specific accessibility role (e.g. "statusbar").
        #[arg(long)]
        role: Option<String>,
        /// Match accessible names by regular expression.
        #[arg(long = "name-regex")]
        name_regex: Option<String>,
        /// What to do when the rule matches.
        #[arg(long, default_value = "suppress")]
        action: ActionArg,
    },
    /// Remove the rule numbered N (as shown by `list`).
    Remove {
        /// 1-based rule number from `oxeye exclusions list`.
        index: usize,
    },
    /// Print the path to the settings file.
    Path,
}

/// CLI mirror of [`oxeye_core::Action`], so the core stays free of any CLI dependency.
#[derive(Clone, Copy, ValueEnum)]
enum ActionArg {
    /// Do not announce at all.
    Suppress,
    /// Announce a shortened summary instead of the full content.
    Summarize,
    /// Announce, but without interrupting in-progress speech.
    LowerPriority,
}

impl From<ActionArg> for Action {
    fn from(arg: ActionArg) -> Self {
        match arg {
            ActionArg::Suppress => Self::Suppress,
            ActionArg::Summarize => Self::Summarize,
            ActionArg::LowerPriority => Self::LowerPriority,
        }
    }
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Exclusions { command } => run_exclusions(command),
    }
}

/// Dispatch an `exclusions` subcommand: load settings, mutate, persist, and report.
fn run_exclusions(command: ExclusionsCommand) -> Result<()> {
    match command {
        ExclusionsCommand::List => {
            let settings = Settings::load().context("loading settings")?;
            println!("{}", oxeye_cli::format_list(&settings));
        }
        ExclusionsCommand::Add {
            app,
            role,
            name_regex,
            action,
        } => {
            let mut settings = Settings::load().context("loading settings")?;
            let rule = ExclusionRule {
                app,
                role,
                name_regex,
                action: action.into(),
            };
            oxeye_cli::add_rule(&mut settings, rule)?;
            settings.save().context("saving settings")?;
            println!("added rule; {} now configured", settings.exclusions.len());
        }
        ExclusionsCommand::Remove { index } => {
            let mut settings = Settings::load().context("loading settings")?;
            let removed = oxeye_cli::remove_rule(&mut settings, index)?;
            settings.save().context("saving settings")?;
            println!(
                "removed rule #{index} ([{}])",
                oxeye_cli::action_label(removed.action)
            );
        }
        ExclusionsCommand::Path => {
            let path = oxeye_core::settings::config_file().context("locating config file")?;
            println!("{}", path.display());
        }
    }
    Ok(())
}
