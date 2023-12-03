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

//! Configuration for git-z.

mod v0_1;
mod v0_2_dev_0;

// NOTE: When you switch to a new version:
//
// - write a new version module,
// - switch the version here,
// - update VERSION below,
// - update the version in `git-z.toml[.sample]`,
// - write a new `impl From<old::Config> for Config` implementations,
// - write a new load_from_* method to load from the previous version,
// - write an updater in `git-z update`,
// - update this message with instructions for next versions.
pub use v0_2_dev_0::{Config, Scopes, Templates, Ticket};

use std::{fs, io, path::PathBuf, process::Command};

use indexmap::IndexMap;
use thiserror::Error;

/// An error that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    #[error("Failed to read {CONFIG_FILE_NAME}")]
    ReadError(#[from] io::Error),
    #[error("Failed to parse {CONFIG_FILE_NAME}")]
    ParseError(#[from] toml::de::Error),
    #[error("The config file is out of date")]
    OutOfDate,
}

/// An error that can occur when building the config file path.
#[derive(Debug, Error)]
pub enum ConfigFileError {
    #[error("Failed to get the Git repo root")]
    RepoRootError(#[from] RepoRootError),
}

/// An error that can occur when getting the Git repo root.
#[derive(Debug, Error)]
pub enum RepoRootError {
    #[error("Failed to run the git command")]
    CannotRunGit(#[from] io::Error),
    #[error("{0}")]
    GitError(String),
    #[error("The output of the git command is not proper UTF-8")]
    EncodingError(#[from] std::string::FromUtf8Error),
}

const CONFIG_FILE_NAME: &str = "git-z.toml";
const VERSION: &str = "0.2-dev.0";

const DEFAULT_TEMPLATE: &str = include_str!("../templates/COMMIT_EDITMSG");

impl Default for Config {
    fn default() -> Self {
        Self {
            version: String::from(VERSION),
            types: IndexMap::from([
                (
                    String::from("feat"),
                    String::from("introduces a new feature"),
                ),
                (String::from("fix"), String::from("patches a bug")),
            ]),
            scopes: None,
            ticket: Ticket {
                prefixes: vec![String::from("")],
            },
            templates: Templates {
                commit: String::from(DEFAULT_TEMPLATE),
            },
        }
    }
}

impl Config {
    /// Loads the configuration the repo or fallbacks to the default.
    pub fn load() -> Result<Self, LoadError> {
        let config = match fs::read_to_string(config_file()?) {
            Ok(config) => toml::from_str(&config)?,

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Self::default(),
                _ => return Err(LoadError::ReadError(error)),
            },
        };

        if config.version == VERSION {
            Ok(config)
        } else {
            Err(LoadError::OutOfDate)
        }
    }
}

fn config_file() -> Result<PathBuf, ConfigFileError> {
    Ok(repo_root()?.join(CONFIG_FILE_NAME))
}

fn repo_root() -> Result<PathBuf, RepoRootError> {
    let git_rev_parse = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if git_rev_parse.status.success() {
        let repo_root = String::from_utf8(git_rev_parse.stdout)?;
        Ok(PathBuf::from(repo_root.trim()))
    } else {
        let git_error = String::from_utf8(git_rev_parse.stderr)?;
        Err(RepoRootError::GitError(git_error.trim().to_owned()))
    }
}
