// git-z - A Git extension to go beyond.
// Copyright (C) 2023-2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
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

//! The `update` subcommand.

use clap::Parser;
use eyre::{bail, Result};
use inquire::Confirm;
use thiserror::Error;

use crate::{
    config::{
        updater::{AskForTicket, ConfigUpdater, Init},
        VERSION,
    },
    error, hint, success,
};

use super::helpers::ensure_in_git_worktree;

/// The update command.
#[derive(Debug, Parser)]
pub struct Update;

/// Usage errors of `git z init`.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// The version of the current configuration is not supported.
    #[error("Unsupported configuration version {version}")]
    UnsupportedVersion {
        /// The unsupported version.
        version: String,
    },
    /// The version of the configuration is an old development one.
    #[error("Unsupported development configuration version {version}")]
    UnsupportedDevelopmentVersion {
        /// The unsupported development version.
        version: String,
        /// The release of `git-z` supporting updates from this version.
        gitz_version: String,
    },
}

impl super::Command for Update {
    fn run(&self) -> Result<()> {
        ensure_in_git_worktree()?;

        let updater = ConfigUpdater::load()?;

        match updater.config_version() {
            VERSION => success!("The configuration is already up to date."),
            "0.1" => update_from_v0_1(updater)?,
            version @ ("0.2-dev.0" | "0.2-dev.1" | "0.2-dev.2"
            | "0.2-dev.3") => {
                bail!(UpdateError::UnsupportedDevelopmentVersion {
                    version: version.to_owned(),
                    gitz_version: String::from("0.2.0"),
                })
            }
            version => bail!(UpdateError::UnsupportedVersion {
                version: version.to_owned()
            }),
        }

        Ok(())
    }
}

/// Updates the configuration from version 0.1.
fn update_from_v0_1(updater: ConfigUpdater<Init>) -> Result<()> {
    let switch_scopes_to_any = ask_scopes_any(&updater)?;
    let ask_for_ticket = ask_ticket_management()?;

    let empty_prefix_to_hash = match ask_for_ticket {
        AskForTicket::Ask { .. } => ask_empty_prefix_to_hash(&updater)?,
        AskForTicket::DontAsk => false,
    };

    updater
        .update_from_v0_1(
            switch_scopes_to_any,
            ask_for_ticket,
            empty_prefix_to_hash,
        )?
        .save()?;

    success!("The configuration has been updated.");
    Ok(())
}

/// Asks the user whether to switch the scopes to `"any"`.
fn ask_scopes_any(updater: &ConfigUpdater<Init>) -> Result<bool> {
    if updater.parsed_config().scopes.is_none() {
        return Ok(false);
    }

    hint! {"

        It is now possible to accept any arbitrary scope instead of a pre-defined list.
    "};

    Ok(Confirm::new(
        "Do you want to accept any scope instead of a pre-defined list?",
    )
    .with_help_message("Answer no to keep the current behaviour (default)")
    .with_default(false)
    .prompt()?)
}

/// Asks the user whether a ticket should be asked for / required.
fn ask_ticket_management() -> Result<AskForTicket> {
    hint! {"

        The ticket / issue number management has been updated. It is now possible to:

        - ask for a required ticket number (as before),
        - ask for an optional ticket number,
        - do not ask for any ticket number.
    "};

    let ask_for_ticket = Confirm::new(
        "Should the committer be proposed to enter a ticket number?",
    )
    .with_default(true)
    .prompt()?;

    let ask_for_ticket = if ask_for_ticket {
        let require = Confirm::new("Should the ticket number be required?")
            .with_default(true)
            .prompt()?;

        AskForTicket::Ask { require }
    } else {
        AskForTicket::DontAsk
    };

    Ok(ask_for_ticket)
}

/// Asks the user whether to convert an empty ticket prefix to `#`.
fn ask_empty_prefix_to_hash(updater: &ConfigUpdater<Init>) -> Result<bool> {
    if updater.parsed_config().ticket.is_none() {
        return Ok(false);
    }

    hint! {r##"

        "#" is now properly handled as a ticket prefix. This means that if \"#\" is
        part of your prefix list, a ticket number `#23` would be properly extracted
        from a branch named `feature/23-name`.
    "##};

    Ok(
        Confirm::new("Should any existing empty value in `ticket.prefixes` be replaced by \"#\"?")
            .with_help_message("This will also remove any `#` prefix before `{{ ticket }}` in your commit template")
            .with_default(true)
            .prompt()?,
    )
}
