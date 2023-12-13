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

use clap::Parser;
use eyre::Result;
use inquire::Confirm;

use crate::{
    config::{
        updater::{AskForTicket, ConfigUpdater, Init},
        VERSION,
    },
    error, hint, success,
};

/// The update command.
#[derive(Debug, Parser)]
pub struct Update;

impl super::Command for Update {
    fn run(&self) -> Result<()> {
        let updater = ConfigUpdater::load()?;

        match updater.config_version() {
            VERSION => success!("The configuration is already up to date."),
            "0.2-dev.1" => update_from_v0_2_dev_1(updater)?,
            "0.2-dev.0" => update_from_v0_2_dev_0(updater)?,
            "0.1" => update_from_v0_1(updater)?,
            version => unknown_version(version),
        }

        Ok(())
    }
}

fn update_from_v0_2_dev_1(updater: ConfigUpdater<Init>) -> Result<()> {
    let empty_prefix_to_hash = ask_empty_prefix_to_hash()?;

    updater
        .update_from_v0_2_dev_1(empty_prefix_to_hash)?
        .save()?;

    success!("The configuration has been updated.");
    Ok(())
}

fn update_from_v0_2_dev_0(updater: ConfigUpdater<Init>) -> Result<()> {
    let ticket = ask_ticket_management()?;
    let empty_prefix_to_hash = ask_empty_prefix_to_hash()?;

    updater
        .update_from_v0_2_dev_0(ticket, empty_prefix_to_hash)?
        .save()?;

    success!("The configuration has been updated.");
    Ok(())
}

fn update_from_v0_1(updater: ConfigUpdater<Init>) -> Result<()> {
    let ask_for_ticket = ask_ticket_management()?;
    let empty_prefix_to_hash = ask_empty_prefix_to_hash()?;

    updater
        .update_from_v0_1(ask_for_ticket, empty_prefix_to_hash)?
        .save()?;

    success!("The configuration has been updated.");
    Ok(())
}

fn ask_ticket_management() -> Result<AskForTicket> {
    hint!("");
    hint!("The ticket / issue number management has been updated. It is now possible to:");
    hint!("");
    hint!("- ask for a required ticket number (as before),");
    hint!("- ask for an optional ticket number,");
    hint!("- do not ask for any ticket number.");
    hint!("");

    let ask_for_ticket = Confirm::new(
        "Should the commiter be proposed to enter a ticket number?",
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

fn ask_empty_prefix_to_hash() -> Result<bool> {
    hint!("");
    hint!("\"#\" is now properly handled as a ticket prefix. This means that if \"#\" is ");
    hint!("part of your prefix list, a ticket number `#23` would be properly extracted ");
    hint!("from a branch named `feature/23-name`.");
    hint!("");
    Ok(
        Confirm::new("Should any existing empty value in `ticket.prefixes` be replaced by \"#\"?")
            .with_help_message("This will also remove any `#` prefix before `{{ ticket }}` in your commit template")
            .with_default(true)
            .prompt()?,
    )
}

fn unknown_version(version: &str) {
    error!("Unkown config version {version}.");
    hint!("Your config file may have been created by a more recent version of git-z.");
    std::process::exit(1);
}
