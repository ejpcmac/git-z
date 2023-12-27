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

use std::error::Error;

use clap::Parser;
use eyre::Result;

use self::{
    commit::{Commit, CommitError},
    helpers::NotInGitWorktree,
    init::{Init, InitError},
    update::{Update, UpdateError},
};
use crate::{
    config::{self, updater, FromTomlError, CONFIG_FILE_NAME},
    error, hint,
};

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
        match error {
            NotInGitWorktree::CannotRunGit(os_error) => {
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
    } else if let Some(config::LoadError::InvalidConfig(from_toml_error)) =
        error.downcast_ref::<config::LoadError>()
    {
        handle_from_toml_error(from_toml_error)
    } else if let Some(updater::LoadError::InvalidConfig(from_toml_error)) =
        error.downcast_ref::<updater::LoadError>()
    {
        handle_from_toml_error(from_toml_error)
    } else if let Some(error) = error.downcast_ref::<CommitError>() {
        match error {
            CommitError::Git { status_code } => {
                #[allow(clippy::exit)]
                std::process::exit(status_code.unwrap_or(1_i32));
            }
            CommitError::Template(tera_error) => {
                error!("{tera_error} from the configuration.");

                if let Some(parse_error) = tera_error.source() {
                    hint!("\n{parse_error}\n");
                }

                #[allow(clippy::exit)]
                std::process::exit(1);
            }
        }
    } else if let Some(error) = error.downcast_ref::<InitError>() {
        match error {
            InitError::ExistingConfig => {
                error!("{error}.");
                hint!("You can force the command by running `git z init -f`.");
            }
        }

        #[allow(clippy::exit)]
        std::process::exit(1);
    } else if let Some(error) = error.downcast_ref::<UpdateError>() {
        match error {
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

fn handle_from_toml_error(error: &FromTomlError) -> ! {
    match error {
        FromTomlError::UnsupportedVersion(_) => {
            error!("{error}.");
            hint!("The {CONFIG_FILE_NAME} may have been created by a newer version of git-z.");
        }
        FromTomlError::ParseError(parse_error) => {
            error!("Invalid configuration in {CONFIG_FILE_NAME}.");
            hint!("\n{parse_error}");
        }
    }

    #[allow(clippy::exit)]
    std::process::exit(1);
}
