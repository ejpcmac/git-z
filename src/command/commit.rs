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
use eyre::{bail, eyre, Result};
use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;
use serde::Serialize;
use tera::{Context, Tera};

use crate::config::Config;

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
    ticket: String,
}

impl super::Command for Commit {
    fn run(&self) -> Result<()> {
        let config = Config::load()?;

        let commit_message = CommitMessage::run_wizard(&config)?;
        let context = Context::from_serialize(commit_message)?;
        let message = Tera::one_off(&config.template, &context, false)?;

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
    let choice = Select::new("Commit type", config.types.clone())
        .with_page_size(PAGE_SIZE)
        .with_formatter(&|choice| remove_type_description(choice.value))
        .prompt()?;
    Ok(remove_type_description(&choice))
}

fn ask_scope(config: &Config) -> Result<Option<String>> {
    if config.scopes.is_empty() {
        Ok(None)
    } else {
        let help_message = format!(
            "{}, {}, {}",
            "↑↓ to move, enter to select, type to filter",
            "ESC to leave empty",
            "update `commits.toml` to add new scopes"
        );

        Ok(Select::new("Scope", config.scopes.clone())
            .with_help_message(&help_message)
            .with_page_size(PAGE_SIZE)
            .prompt_skippable()?)
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

fn ask_ticket(config: &Config) -> Result<String> {
    let placeholder = ticket_placeholder(config)?;
    let mut ticket = Text::new("Issue / ticket number")
        .with_placeholder(&placeholder)
        .with_validator(validate_ticket);

    let ticket_from_branch = get_ticket_from_branch(config)?;
    ticket.initial_value = ticket_from_branch.as_deref();

    Ok(ticket.prompt()?)
}

fn get_ticket_from_branch(config: &Config) -> Result<Option<String>> {
    let regex = ticket_regex(config)?;
    Ok(Regex::new(&regex)?
        .captures(&get_current_branch()?)
        .map(|captures| captures[0].to_owned()))
}

fn get_current_branch() -> Result<String> {
    let git_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;
    assert!(git_branch.status.success());
    Ok(String::from_utf8(git_branch.stdout)?)
}

fn remove_type_description(choice: &str) -> String {
    // NOTE(unwrap): Even an empty string will contain at list one split, so the
    // only call to next will always return Some(value).
    #[allow(clippy::unwrap_used)]
    choice.split(' ').next().unwrap().to_owned()
}

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
    let regex = ticket_regex(&config)?;
    let placeholder = ticket_placeholder(&config)?;

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

fn ticket_regex(config: &Config) -> Result<String> {
    let valid_prefixes = config
        .ticket_prefixes
        .clone()
        .into_iter()
        .reduce(|acc, prefix| format!("{acc}|{prefix}"))
        .ok_or(eyre!("empty ticket prefix list"))?;

    Ok(format!("(?:{valid_prefixes})\\d+"))
}

fn ticket_placeholder(config: &Config) -> Result<String> {
    config
        .ticket_prefixes
        .iter()
        .map(|prefix| format!("{prefix}XXX"))
        .reduce(|acc, prefix| format!("{acc} or {prefix}"))
        .ok_or(eyre!("empty ticket prefix list"))
}
