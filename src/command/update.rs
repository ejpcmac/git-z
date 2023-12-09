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

use crate::{
    config::{
        updater::{ConfigUpdater, Init},
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
            "0.1" => update_from_v0_1(updater)?,
            version => unknown_version(version),
        }

        Ok(())
    }
}

fn update_from_v0_1(updater: ConfigUpdater<Init>) -> Result<()> {
    updater.update_from_v0_1()?.save()?;
    success!("the configuration has been updated.");
    Ok(())
}

fn unknown_version(version: &str) {
    error!("Unkown config version {version}.");
    hint!("Your config file may have been created by a more recent version of command.");
    std::process::exit(1);
}
