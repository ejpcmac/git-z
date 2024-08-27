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

//! The `init` subcommand.

use std::fs;

use askama::Template;
use clap::Parser;
use eyre::{bail, Result};
use inquire::Select;
use thiserror::Error;

use crate::{config::config_file, hint, success};

use super::helpers::ensure_in_git_worktree;

/// The init command.
#[derive(Debug, Parser)]
pub struct Init {
    /// Use the default configuration.
    #[arg(long, short = 'd')]
    default: bool,
    /// Force the init process.
    #[arg(long, short = 'f')]
    force: bool,
}

/// Usage errors of `git z init`.
#[derive(Debug, Error)]
pub enum InitError {
    /// A configuration already exists.
    #[error("There is already a git-z.toml in the current repository")]
    ExistingConfig,
}

/// Parameters to generate a `git-z.toml`.
#[derive(Debug, Default, Template)]
#[template(path = "git-z.toml.jinja", syntax = "template")]
struct Config {
    /// Whether to ask for a scope.
    scopes: Scopes,
    /// Whether to ask for a ticket.
    ticket: Ticket,
}

/// Whether to ask for a scope.
#[derive(Debug)]
enum Scopes {
    /// Ask for a scope.
    Ask {
        /// What scopes to accept.
        accept: AcceptScopes,
    },
    /// Do not ask for a scope.
    DontAsk,
}

/// The scopes to accept.
#[derive(Debug, Default)]
enum AcceptScopes {
    /// Accept arbitrary scopes.
    #[default]
    Any,
    /// Accept scopes from a list.
    List,
}

/// Whether to ask for a ticket.
#[derive(Debug)]
enum Ticket {
    /// Ask for a ticket.
    Ask {
        /// Whether to require a ticket.
        required: bool,
    },
    /// Do not ask for a ticket.
    DontAsk,
}

impl super::Command for Init {
    fn run(&self) -> Result<()> {
        ensure_in_git_worktree()?;

        let config_file = config_file()?;

        if !self.force && config_file.exists() {
            bail!(InitError::ExistingConfig);
        }

        let config = if self.default {
            Config::default()
        } else {
            Config::run_wizard()?
        };

        fs::write(config_file, format!("{config}\n"))?;

        success!("A git-z.toml has been created!");
        hint!("You can now edit it to adjust the configuration.");

        Ok(())
    }
}

impl Config {
    /// Runs the wizard to fill the parameters for the configuration.
    fn run_wizard() -> Result<Self> {
        Ok(Self {
            scopes: Scopes::run_wizard()?,
            ticket: Ticket::run_wizard()?,
        })
    }
}

impl Scopes {
    /// Runs the wizard for scope configuration.
    fn run_wizard() -> Result<Self> {
        let options = vec![
            "Ask for a scope, accept any",
            "Ask for a scope in a list",
            "Do not ask for a scope",
        ];

        let choice = Select::new("Should git-z ask for a scope?", options)
            .with_starting_cursor(0)
            .prompt()?;

        let choice = match choice {
            "Ask for a scope, accept any" => Self::Ask {
                accept: AcceptScopes::Any,
            },
            "Ask for a scope in a list" => Self::Ask {
                accept: AcceptScopes::List,
            },
            _ => Self::DontAsk,
        };

        Ok(choice)
    }
}

impl Ticket {
    /// Runs the wizard for ticket configuration.
    fn run_wizard() -> Result<Self> {
        let options = vec![
            "Require a ticket number",
            "Ask for an optional ticket number",
            "Do not ask for a ticket number",
        ];

        let choice =
            Select::new("Should git-z ask for a ticket number?", options)
                .with_starting_cursor(1)
                .prompt()?;

        let choice = match choice {
            "Require a ticket number" => Self::Ask { required: true },
            "Ask for an optional ticket number" => {
                Self::Ask { required: false }
            }
            _ => Self::DontAsk,
        };

        Ok(choice)
    }
}

impl Default for Scopes {
    fn default() -> Self {
        Self::Ask {
            accept: AcceptScopes::default(),
        }
    }
}

impl Default for Ticket {
    fn default() -> Self {
        Self::Ask { required: false }
    }
}
