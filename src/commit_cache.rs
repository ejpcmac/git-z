// git-z - A Git extension to go beyond.
// Copyright (C) 2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
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

//! Cache for aborted commits.

use std::{fs, io, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The commit cache.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitCache {
    /// The version of the commit cache.
    pub version: String,
    /// The state of the wizard.
    pub wizard_state: WizardState,
    /// The answers to the wizard questions.
    pub wizard_answers: WizardAnswers,
}

/// The state of the wizard.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum WizardState {
    /// The wizard is not started.
    #[default]
    NotStarted,
    /// The wizard is ongoing.
    Ongoing,
    /// The wizard has completed.
    Completed,
}

/// The cached answers.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WizardAnswers {
    /// The answer for the type.
    pub r#type: Option<String>,
    /// The answer for the scope.
    pub scope: Option<String>,
    /// The answer for the description.
    pub description: Option<String>,
    /// The answer for the breaking change.
    pub breaking_change: Option<String>,
    /// The answer for the ticket.
    pub ticket: Option<String>,
}

/// Errors that can occur when loading the commit cache.
#[derive(Debug, Error)]
pub enum LoadError {
    /// The path of the commit cache cannot be resolved.
    #[error("Failed to get the path of the commit cache file")]
    CommitCacheFile(#[from] CommitCacheFileError),
    /// An error has occurred while reading the commit cache file.
    #[error("Failed to read the commit cache")]
    Read(#[from] io::Error),
}

/// Errors that can occur when saving the commit cache.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The path of the git-z directory cannot be resolved.
    #[error("Failed to get the path of the git-z directory")]
    GitZDir(#[from] GitZDirError),
    /// The path of the commit cache file cannot be resolved.
    #[error("Failed to get the path of the commit cache file")]
    CommitCacheFile(#[from] CommitCacheFileError),
    /// Error while writing the commit cache file.
    #[error("Failed to write the commit cache")]
    Write(#[from] io::Error),
}

/// Errors that can occur when discarding the commit cache.
#[derive(Debug, Error)]
pub enum DiscardError {
    /// The path of the commit cache file cannot be resolved.
    #[error("Failed to get the path of the commit cache file")]
    CommitCacheFile(#[from] CommitCacheFileError),
    /// Error while deleting the commit cache file.
    #[error("Failed to delete the commit cache file")]
    Delete(#[from] io::Error),
}

/// Errors that can occur when parsing the TOML.
#[derive(Debug, Error)]
pub enum FromTomlError {
    /// The version of the commit cache is not supported.
    #[error("Unsupported commit cache version {version}")]
    UnsupportedVersion {
        /// The unsupported version.
        version: String,
    },
    /// The commit cache file cannot be parsed.
    #[error("Failed to parse the commit cache file")]
    ParseError(#[from] toml::de::Error),
}

/// Errors that can occur when building the path of the commit cache file.
#[derive(Debug, Error)]
pub enum CommitCacheFileError {
    /// The path of the git-z directory cannot be resolved.
    #[error("Failed to get the path of the git-z directory")]
    GitZDirError(#[from] GitZDirError),
}

/// Errors that can occur when building the path of the git-z directory.
#[derive(Debug, Error)]
pub enum GitZDirError {
    /// An error has occurred while getting the path of the Git directory.
    #[error("Failed to get the path of the Git directory")]
    GitDirError(#[from] GitDirError),
}

/// Errors that can occur when getting the Git directory.
#[derive(Debug, Error)]
pub enum GitDirError {
    /// The `git` command cannot be run.
    #[error("Failed to run the git command")]
    CannotRunGit(#[from] io::Error),
    /// Git has returned an error.
    #[error("{0}")]
    GitError(String),
    /// The output of the git command is not proper UTF-8.
    #[error("The output of the git command is not proper UTF-8")]
    EncodingError(#[from] std::string::FromUtf8Error),
}

/// A minimal commit cache to get the version.
///
/// The format of the commit cache can evolve with time. It is versioned so that
/// git-z can invalidate any cache produced by an incompatible version.
#[derive(Debug, Serialize, Deserialize)]
struct MinimalCommitCache {
    /// The version of the commit cache.
    version: String,
}

/// The name of the git-z directory.
const GITZ_DIR_NAME: &str = "git-z";

/// The name of the commit cache file.
const COMMIT_CACHE_FILE_NAME: &str = "commit-cache.toml";

/// The current version of the config cache.
const VERSION: &str = "0.1";

impl Default for CommitCache {
    fn default() -> Self {
        Self {
            version: String::from(VERSION),
            wizard_state: WizardState::default(),
            wizard_answers: WizardAnswers::default(),
        }
    }
}

impl CommitCache {
    /// Loads the commit cache of the repo or fallbacks to the default.
    pub fn load() -> Result<Self, LoadError> {
        match fs::read_to_string(commit_cache_file()?) {
            Ok(commit_cache) => {
                let commit_cache = Self::from_toml(&commit_cache)
                    .unwrap_or_else(|_| {
                        // If the existing cache is not usable, let’s discard it
                        // and start from a fresh one.
                        Self::discard().ok();
                        Self::default()
                    });

                Ok(commit_cache)
            }

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Ok(Self::default()),
                _ => Err(LoadError::Read(error)),
            },
        }
    }

    /// Gets the answer for the type.
    pub fn r#type(&self) -> Option<&str> {
        self.wizard_answers.r#type.as_deref()
    }

    /// Gets the answer for the scope.
    pub fn scope(&self) -> Option<&str> {
        self.wizard_answers.scope.as_deref()
    }

    /// Gets the answer for the description.
    pub fn description(&self) -> Option<&str> {
        self.wizard_answers.description.as_deref()
    }

    /// Gets the answer for the breaking change.
    pub fn breaking_change(&self) -> Option<&str> {
        self.wizard_answers.breaking_change.as_deref()
    }

    /// Gets the answer for the ticket.
    pub fn ticket(&self) -> Option<&str> {
        self.wizard_answers.ticket.as_deref()
    }

    /// Resets the commit cache and discards it from the repo.
    pub fn reset(&mut self) -> Result<(), DiscardError> {
        self.wizard_answers = WizardAnswers::default();
        self.wizard_state = WizardState::default();
        Self::discard()
    }

    /// Sets the answer for the type.
    pub fn set_type(&mut self, r#type: &str) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.wizard_answers.r#type = Some(r#type.to_owned());
        self.save()
    }

    /// Sets the answer for the scope.
    pub fn set_scope(&mut self, scope: Option<&str>) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.wizard_answers.scope = scope.map(ToOwned::to_owned);
        self.save()
    }

    /// Sets the answer for the description.
    pub fn set_description(
        &mut self,
        description: &str,
    ) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.wizard_answers.description = Some(description.to_owned());
        self.save()
    }

    /// Sets the answer for the breaking change.
    pub fn set_breaking_change(
        &mut self,
        breaking_change: Option<&str>,
    ) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.wizard_answers.breaking_change =
            breaking_change.map(ToOwned::to_owned);
        self.save()
    }

    /// Sets the answer for the ticket.
    pub fn set_ticket(
        &mut self,
        ticket: Option<&str>,
    ) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.wizard_answers.ticket = ticket.map(ToOwned::to_owned);
        self.save()
    }

    /// Marks the wizard as ongoing.
    pub fn mark_wizard_as_ongoing(&mut self) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Ongoing;
        self.save()
    }

    /// Marks the wizard as completed.
    pub fn mark_wizard_as_completed(&mut self) -> Result<(), SaveError> {
        self.wizard_state = WizardState::Completed;
        self.save()
    }

    /// Discards the current commit cache from the repo.
    pub fn discard() -> Result<(), DiscardError> {
        fs::remove_file(commit_cache_file()?)?;
        Ok(())
    }

    /// Saves the commit cache to the repo.
    // NOTE(allow): The function cannot actually panic (see note on expect).
    #[allow(clippy::unwrap_in_result, clippy::missing_panics_doc)]
    fn save(&self) -> Result<(), SaveError> {
        // NOTE(allow): We control the format, so a serialisation error would
        // be a bug in the code, not an error.
        #[allow(clippy::expect_used)]
        let commit_cache = toml::to_string(self)
            .expect("Failed to serialise the commit cache");

        fs::create_dir_all(gitz_dir()?)?;
        fs::write(commit_cache_file()?, commit_cache)?;
        Ok(())
    }

    /// Builds a commit cache from its TOML representation.
    fn from_toml(toml: &str) -> Result<Self, FromTomlError> {
        let minimal_cache: MinimalCommitCache = toml::from_str(toml)?;

        if minimal_cache.version.as_str() == VERSION {
            Ok(toml::from_str(toml)?)
        } else {
            Err(FromTomlError::UnsupportedVersion {
                version: minimal_cache.version,
            })
        }
    }
}

/// Returns the path of the commit cache file.
fn commit_cache_file() -> Result<PathBuf, CommitCacheFileError> {
    Ok(gitz_dir()?.join(COMMIT_CACHE_FILE_NAME))
}

/// Returns the path of the git-z directory.
fn gitz_dir() -> Result<PathBuf, GitZDirError> {
    Ok(git_dir()?.join(GITZ_DIR_NAME))
}

/// Returns the path of the Git directory.
fn git_dir() -> Result<PathBuf, GitDirError> {
    let git_rev_parse = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if git_rev_parse.status.success() {
        let repo_root = String::from_utf8(git_rev_parse.stdout)?;
        Ok(PathBuf::from(repo_root.trim()))
    } else {
        let git_error = String::from_utf8(git_rev_parse.stderr)?;
        Err(GitDirError::GitError(git_error.trim().to_owned()))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::pedantic, clippy::restriction)]

    use indoc::formatdoc;

    use super::*;

    #[test]
    fn toml_representation_for_default() {
        let commit_cache = CommitCache::default();

        assert_eq!(
            toml::to_string(&commit_cache).unwrap(),
            formatdoc! {r##"
                version = "{VERSION}"
                wizard_state = "not_started"

                [wizard_answers]
            "##}
        );
    }

    #[test]
    fn toml_representation_for_ongoing() {
        let commit_cache = CommitCache {
            version: String::from(VERSION),
            wizard_state: WizardState::Ongoing,
            wizard_answers: WizardAnswers {
                r#type: Some(String::from("feat")),
                scope: None,
                description: Some(String::from("some description")),
                breaking_change: None,
                ticket: Some(String::from("#23")),
            },
        };

        assert_eq!(
            toml::to_string(&commit_cache).unwrap(),
            formatdoc! {r##"
                version = "{VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "feat"
                description = "some description"
                ticket = "#23"
            "##}
        );
    }

    #[test]
    fn toml_representation_for_completed() {
        let commit_cache = CommitCache {
            version: String::from(VERSION),
            wizard_state: WizardState::Completed,
            wizard_answers: WizardAnswers::default(),
        };

        assert_eq!(
            toml::to_string(&commit_cache).unwrap(),
            formatdoc! {r##"
                version = "{VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##}
        );
    }
}