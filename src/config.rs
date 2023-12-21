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

pub mod updater;

mod v0_1;
mod v0_2_dev_0;
mod v0_2_dev_1;
mod v0_2_dev_2;
mod v0_2_dev_3;

// NOTE: When you switch to a new version:
//
// - write a new version module,
// - switch the version here,
// - update VERSION below,
// - update the version in `templates/git-z.toml.jinja`,
// - update the `impl From<old::Config> for Config` implementations,
// - write a new `impl From<previous::Config> for Config` implementation,
// - handle the previous config in `Config::load`,
// - write an updater in `ConfigUpdater`,
// - update the previous updaters as well,
// - update `git z update`.
pub use v0_2_dev_3::{Config, Scopes, Templates, Ticket};

use std::{fs, io, path::PathBuf, process::Command};

use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An error that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    #[error("Failed to read {CONFIG_FILE_NAME}")]
    ReadError(#[from] io::Error),
    #[error("Invalid configuration in {CONFIG_FILE_NAME}")]
    InvalidConfig(#[from] FromTomlError),
}

/// An error that can occur when parsing the TOML.
#[derive(Debug, Error)]
pub enum FromTomlError {
    #[error("Configuration version {0} is not supported")]
    UnsupportedVersion(String),
    #[error("Failed to parse into a valid configuration")]
    ParseError(#[from] toml::de::Error),
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

/// A minimal configuration to get the version.
///
/// The configuration format for git-z can evolve with time. It is versioned for
/// this purpose, so that git-z is able to select the proper parser. This struct
/// allows to parse any configuration as long as it contains a version field.
#[derive(Debug, Serialize, Deserialize)]
struct MinimalConfig {
    /// The version of the configuration.
    version: String,
}

/// The name of the configuration file.
pub const CONFIG_FILE_NAME: &str = "git-z.toml";
/// The current version of the configuration file.
pub const VERSION: &str = "0.2-dev.3";

const DEFAULT_TEMPLATE: &str = include_str!("../templates/COMMIT_EDITMSG");

impl Default for Config {
    fn default() -> Self {
        let default_types = indexmap! {
            "feat" => "adds a new feature in the code",
            "sec" => "patches a security issue",
            "fix" => "patches a code bug",
            "perf" => "enhances the performance, without adding a new feature",
            "refactor" => "refactors the code",
            "test" => "adds, updates or removes tests only",
            "docs" => "updates the documentation only",
            "style" => "updates the style, like running clang-format or changing headers",
            "deps" => "adds, updates or removes external dependencies",
            "build" => "updates the build system or build scripts",
            "env" => "updates the development environment",
            "ide" => "updates the IDE configuration",
            "ci" => "updates the CI configuration",
            "revert" => "reverts a previous commit",
            "chore" => "updates something that is not covered by any other type",
            "wip" => "work in progress / to be rebased and squashed later",
            "debug" => "commit used for debugging purposes, not to be integrated",
        };

        Self {
            version: String::from(VERSION),
            types: default_types
                .into_iter()
                .map(|(key, value)| (String::from(key), String::from(value)))
                .collect(),
            scopes: Some(Scopes::Any),
            ticket: None,
            templates: Templates {
                commit: String::from(DEFAULT_TEMPLATE),
            },
        }
    }
}

impl Config {
    /// Loads the configuration the repo or fallbacks to the default.
    pub fn load() -> Result<Self, LoadError> {
        match fs::read_to_string(config_file()?) {
            Ok(config) => Ok(Self::from_toml(&config)?),

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Ok(Self::default()),
                _ => Err(LoadError::ReadError(error)),
            },
        }
    }

    /// Builds the configuration from its TOML representation.
    pub fn from_toml(toml: &str) -> Result<Self, FromTomlError> {
        let minimal_config: MinimalConfig = toml::from_str(toml)?;

        match minimal_config.version.as_str() {
            VERSION => Ok(toml::from_str(toml)?),
            "0.2-dev.2" => {
                let config: v0_2_dev_2::Config = toml::from_str(toml)?;
                Ok(config.into())
            }
            "0.2-dev.1" => {
                let config: v0_2_dev_1::Config = toml::from_str(toml)?;
                Ok(config.into())
            }
            "0.2-dev.0" => {
                let config: v0_2_dev_0::Config = toml::from_str(toml)?;
                Ok(config.into())
            }
            "0.1" => {
                let config: v0_1::Config = toml::from_str(toml)?;
                Ok(config.into())
            }
            version => {
                Err(FromTomlError::UnsupportedVersion(version.to_owned()))
            }
        }
    }
}

/// Returns the path of the configuration file.
pub fn config_file() -> Result<PathBuf, ConfigFileError> {
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

impl From<v0_2_dev_2::Config> for Config {
    fn from(old: v0_2_dev_2::Config) -> Self {
        Self {
            version: old.version,
            types: old.types,
            scopes: old.scopes.map(Into::into),
            ticket: old.ticket.map(Into::into),
            templates: old.templates.into(),
        }
    }
}

impl From<v0_2_dev_2::Scopes> for Scopes {
    fn from(old: v0_2_dev_2::Scopes) -> Self {
        match old {
            v0_2_dev_2::Scopes::List { list } => Self::List { list },
        }
    }
}

impl From<v0_2_dev_2::Ticket> for Ticket {
    fn from(old: v0_2_dev_2::Ticket) -> Self {
        Self {
            required: old.required,
            prefixes: old.prefixes,
        }
    }
}

impl From<v0_2_dev_2::Templates> for Templates {
    fn from(old: v0_2_dev_2::Templates) -> Self {
        Self { commit: old.commit }
    }
}

impl From<v0_2_dev_1::Config> for Config {
    fn from(old: v0_2_dev_1::Config) -> Self {
        Self {
            version: old.version,
            types: old.types,
            scopes: old.scopes.map(Into::into),
            ticket: old.ticket.map(Into::into),
            templates: old.templates.into(),
        }
    }
}

impl From<v0_2_dev_1::Scopes> for Scopes {
    fn from(old: v0_2_dev_1::Scopes) -> Self {
        match old {
            v0_2_dev_1::Scopes::List { list } => Self::List { list },
        }
    }
}

impl From<v0_2_dev_1::Ticket> for Ticket {
    fn from(old: v0_2_dev_1::Ticket) -> Self {
        Self {
            required: old.required,
            prefixes: old.prefixes,
        }
    }
}

impl From<v0_2_dev_1::Templates> for Templates {
    fn from(old: v0_2_dev_1::Templates) -> Self {
        Self { commit: old.commit }
    }
}

impl From<v0_2_dev_0::Config> for Config {
    fn from(old: v0_2_dev_0::Config) -> Self {
        Self {
            version: old.version,
            types: old.types,
            scopes: old.scopes.map(Into::into),
            ticket: Some(Ticket {
                required: true,
                prefixes: old.ticket.prefixes,
            }),
            templates: Templates {
                commit: old.templates.commit,
            },
        }
    }
}

impl From<v0_2_dev_0::Scopes> for Scopes {
    fn from(old: v0_2_dev_0::Scopes) -> Self {
        match old {
            v0_2_dev_0::Scopes::List { list } => Self::List { list },
        }
    }
}

impl From<v0_1::Config> for Config {
    fn from(old: v0_1::Config) -> Self {
        Self {
            version: old.version,
            types: split_types_and_docs(&old.types),
            scopes: Some(Scopes::List { list: old.scopes }),
            ticket: Some(Ticket {
                required: true,
                prefixes: old.ticket_prefixes,
            }),
            templates: Templates {
                commit: old.template,
            },
        }
    }
}

fn split_types_and_docs(types: &[String]) -> IndexMap<String, String> {
    types
        .iter()
        .map(AsRef::as_ref)
        .map(split_type_and_doc)
        .collect()
}

fn split_type_and_doc(type_and_doc: &str) -> (String, String) {
    let mut split = type_and_doc.splitn(2, ' ');
    let ty = split.next().unwrap_or_default().to_owned();
    let doc = split.next().unwrap_or_default().trim().to_owned();
    (ty, doc)
}
