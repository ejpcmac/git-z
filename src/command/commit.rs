// git-z - A Git extension to go beyond.
// Copyright (C) 2023 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::process::Command;

use clap::Parser;
use eyre::{bail, ensure, eyre, Context as _, Result};
use indexmap::IndexMap;
use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;
use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    command::helpers::load_config,
    config::{Config, Scopes, Ticket},
};

const PAGE_SIZE: usize = 15;

/// The commit command.
#[derive(Debug, Parser)]
pub struct Commit {
    /// Print the commit message instead of calling `git commit`.
    #[arg(long)]
    print_only: bool,
    /// Extra arguments to be passed to `git commit`.
    #[arg(last = true)]
    extra_args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CommitMessage {
    r#type: String,
    scope: Option<String>,
    description: String,
    breaking_change: Option<String>,
    ticket: Option<String>,
}

impl super::Command for Commit {
    fn run(&self) -> Result<()> {
        let config = load_config()?;

        let commit_message = CommitMessage::run_wizard(&config)?;
        let context = Context::from_serialize(commit_message)?;
        let message = Tera::one_off(&config.templates.commit, &context, false)?;

        if self.print_only {
            println!("{message}");
        } else {
            let status = Command::new("git")
                .args(["commit", "-em", &message])
                .args(&self.extra_args)
                .status()?;

            if !status.success() {
                bail!("Git commit failed");
            }
        }

        Ok(())
    }
}

impl CommitMessage {
    fn run_wizard(config: &Config) -> Result<Self> {
        Ok(Self {
            r#type: ask_type(config)?,
            scope: ask_scope(config)?,
            description: ask_description()?,
            breaking_change: ask_breaking_change()?,
            ticket: ask_ticket(config)?,
        })
    }
}

fn ask_type(config: &Config) -> Result<String> {
    let choice = Select::new("Commit type", format_types(&config.types))
        .with_page_size(PAGE_SIZE)
        .with_formatter(&|choice| remove_type_description(choice.value))
        .prompt()?;
    Ok(remove_type_description(&choice))
}

fn ask_scope(config: &Config) -> Result<Option<String>> {
    match config.scopes {
        None => Ok(None),

        Some(Scopes::Any) => Ok(Text::new("Scope")
            .with_help_message("Press ESC or leave empty to omit the scope.")
            .prompt_skippable()?
            .filter(|s| !s.is_empty())),

        Some(Scopes::List { ref list }) => {
            let help_message = format!(
                "{}, {}, {}",
                "↑↓ to move, enter to select, type to filter",
                "ESC to leave empty",
                "update `commits.toml` to add new scopes"
            );

            Ok(Select::new("Scope", list.clone())
                .with_help_message(&help_message)
                .with_page_size(PAGE_SIZE)
                .prompt_skippable()?)
        }
    }
}

fn ask_description() -> Result<String> {
    let placeholder =
        "describe your change with a short description (5-50 characters)";
    let message = "You will be able to add a long description to your commit in an editor later.";

    Ok(Text::new("Short description")
        .with_placeholder(placeholder)
        .with_help_message(message)
        .with_validator(validate_description)
        .prompt()?)
}

fn ask_breaking_change() -> Result<Option<String>> {
    Ok(Text::new("BREAKING CHANGE")
        .with_placeholder("Summary of the breaking change.")
        .with_help_message(
            "Press ESC or leave empty if there are no breaking changes.",
        )
        .prompt_skippable()?
        .filter(|s| !s.is_empty()))
}

fn ask_ticket(config: &Config) -> Result<Option<String>> {
    match config.ticket {
        None => Ok(None),
        Some(Ticket {
            ref required,
            ref prefixes,
        }) => {
            let placeholder = ticket_placeholder(prefixes)?;
            let ticket_from_branch = get_ticket_from_branch(prefixes)?;

            let mut ticket = Text::new("Issue / ticket number")
                .with_placeholder(&placeholder)
                .with_validator(validate_ticket);
            ticket.initial_value = ticket_from_branch.as_deref();

            let ticket = if *required {
                Some(ticket.prompt()?)
            } else {
                ticket
                    .with_help_message(
                        "Press ESC to omit the ticket reference.",
                    )
                    .prompt_skippable()?
            };

            Ok(ticket)
        }
    }
}

fn get_ticket_from_branch(prefixes: &[String]) -> Result<Option<String>> {
    // Replace `#` with an empty string in the regex, as we want to match
    // branches like `feature/23-name` when `#` is a valid prefix like for
    // GitHub or GitLab issues.
    let regex = ticket_regex(prefixes).replace('#', "");

    let ticket = Regex::new(&regex)
        .wrap_err("Impossible to build a regex from the list of prefixes")?
        .captures(&get_current_branch()?)
        .map(|captures| {
            // NOTE(indexing): Capture group 0 always corresponds to an implicit
            // unnamed group that includes the entire match.
            #[allow(clippy::indexing_slicing)]
            captures[0].to_owned()
        })
        .map(|ticket| {
            // NOTE(unwrap): This regex is known to be valid.
            #[allow(clippy::unwrap_used)]
            let regex = &Regex::new(r"^\d+$").unwrap();

            // If one of the valid prefixes is `#` and the matched ticket ID is
            // only made of numbers, we are in the GitHub / GitLab style, so
            // let’s add a `#` as a prefix to the ticket ID.
            if prefixes.contains(&String::from("#")) && regex.is_match(&ticket)
            {
                format!("#{ticket}")
            } else {
                ticket
            }
        });

    Ok(ticket)
}

fn get_current_branch() -> Result<String> {
    let git_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    ensure!(
        git_branch.status.success(),
        "Failed to run `git branch --show-current`"
    );

    Ok(String::from_utf8(git_branch.stdout)?)
}

fn format_types(types: &IndexMap<String, String>) -> Vec<String> {
    let Some(max_type_len) = types.keys().map(String::len).max() else {
        return vec![];
    };

    types
        .iter()
        .map(|(ty, doc)| {
            let padding = " ".repeat(max_type_len - ty.len());
            format!("{ty}{padding}  {doc}")
        })
        .collect()
}

fn remove_type_description(choice: &str) -> String {
    // NOTE(unwrap): Even an empty string will contain at list one split, so the
    // only call to next will always return Some(value).
    #[allow(clippy::unwrap_used)]
    choice.split(' ').next().unwrap().to_owned()
}

#[allow(clippy::unnecessary_wraps)]
fn validate_description(
    description: &str,
) -> Result<Validation, CustomUserError> {
    // NOTE(unwrap): We know from the first condition that description.len() >
    // 0, so there is at least one character in the string. Hence,
    // description.chars().next() in the third condition will always return
    // Some(value).
    #[allow(clippy::unwrap_used)]
    if description.len() < 5 {
        Ok(Validation::Invalid(
            "The description must be longer than 5 characters".into(),
        ))
    } else if description.len() > 50 {
        Ok(Validation::Invalid(
            "The description must not be longer than 50 characters".into(),
        ))
    } else if description.chars().next().unwrap().is_uppercase() {
        Ok(Validation::Invalid(
            "The description must start in lowercase".into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}

fn validate_ticket(ticket: &str) -> Result<Validation, CustomUserError> {
    let config = Config::load()?;
    let prefixes = &config
        .ticket
        .ok_or(eyre!("no ticket prefix list"))?
        .prefixes;

    let regex = ticket_regex(prefixes);
    let placeholder = ticket_placeholder(prefixes)?;

    if Regex::new(&format!("^{regex}$"))?.is_match(ticket) {
        Ok(Validation::Valid)
    } else {
        Ok(Validation::Invalid(
            format!(
                "The issue / ticket number must be in the form {placeholder}"
            )
            .into(),
        ))
    }
}

fn ticket_regex(prefixes: &[String]) -> String {
    let prefixes = prefixes.join("|");
    format!("(?:{prefixes})\\d+")
}

fn ticket_placeholder(prefixes: &[String]) -> Result<String> {
    prefixes
        .iter()
        .map(|prefix| format!("{prefix}XXX"))
        .reduce(|acc, prefix| format!("{acc} or {prefix}"))
        .ok_or(eyre!("empty ticket prefix list"))
}
