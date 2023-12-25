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

mod commit;
mod helpers;
mod init;
mod update;

use clap::Parser;
use eyre::Result;

use self::{
    commit::Commit,
    helpers::NotInGitWorktree,
    init::{Init, InitError},
    update::{Update, UpdateError},
};
use crate::{error, hint};

/// A Git extension to go beyond.
#[derive(Debug, Parser)]
#[command(author, version = env!("VERSION_WITH_GIT"))]
pub enum GitZ {
    /// Initialises the configuration.
    Init(Init),
    /// Runs the commit wizard.
    Commit(Commit),
    /// Updates the configuration.
    Update(Update),
}

trait Command {
    /// Runs the command.
    fn run(&self) -> Result<()>;
}

impl GitZ {
    /// Runs git-z.
    pub fn run() -> Result<()> {
        let result = match Self::parse() {
            Self::Init(init) => init.run(),
            Self::Commit(commit) => commit.run(),
            Self::Update(update) => update.run(),
        };

        match result {
            Err(error) => handle_errors(error),
            Ok(()) => Ok(()),
        }
    }
}

fn handle_errors(error: color_eyre::Report) -> Result<()> {
    if let Some(error) = error.downcast_ref::<NotInGitWorktree>() {
        match *error {
            NotInGitWorktree::CannotRunGit(ref os_error) => {
                error!("{error}.");
                hint!("The OS reports: {os_error}.");
            }
            NotInGitWorktree::NotInRepo => {
                error!("{error}.");
                hint!("You can initialise a Git repository by running `git init`.");
            }
            NotInGitWorktree::NotInWorktree => {
                error!("{error}.");
                hint!("You seem to be inside a Git repository, but not in a worktree.");
            }
        }

        #[allow(clippy::exit)]
        std::process::exit(1);
    } else if let Some(error) = error.downcast_ref::<InitError>() {
        match *error {
            InitError::ExistingConfig => {
                error!("{error}.");
                hint!("You can force the command by running `git z init -f`.");
            }
        }

        #[allow(clippy::exit)]
        std::process::exit(1);
    } else if let Some(error) = error.downcast_ref::<UpdateError>() {
        match *error {
            UpdateError::UnknownVersion { .. } => {
                error!("{error}.");
                hint!("Your config file may have been created by a more recent version of git-z.");
            }
        }

        #[allow(clippy::exit)]
        std::process::exit(1);
    } else {
        Err(error)
    }
}
