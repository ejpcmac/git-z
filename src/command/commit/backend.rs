// git-z - A Git extension to go beyond.
// Copyright (C) 2023-2025 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
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

//! Backend for the `commit` subcommand.

use std::{io, process::Command};

use thiserror::Error;

use crate::tracing::LogResult as _;

/// A commit backend.
pub trait Backend {
    /// Calls the backend.
    fn call(&self, commit_message: &str) -> Result<(), BackendError>;
}

/// Errors that can occur when running the backend command.
#[derive(Debug, Error)]
pub enum BackendError {
    /// The backend command cannot be run.
    #[error("Failed to run `{command}`")]
    CannotRun {
        /// The command that cannot be run.
        command: String,
        /// The OS error.
        os_error: io::Error,
    },
    /// The backend command has returned an error.
    #[error("The backend command has returned an error")]
    ExecutionError {
        /// The status code returned by the command.
        status_code: Option<i32>,
    },
}

/// A backend using `git commit -em "$message"`.
pub struct GitBackend {
    /// Extra arguments to pass to `git commit`.
    extra_args: Vec<String>,
}

impl GitBackend {
    /// Builds a new Git backend.
    #[tracing::instrument(name = "new_git_backend", level = "trace", skip_all)]
    pub fn new(extra_args: &[String]) -> Self {
        Self {
            extra_args: extra_args.to_owned(),
        }
    }
}

impl Backend for GitBackend {
    #[tracing::instrument(name = "git_backend", level = "trace", skip_all)]
    fn call(&self, commit_message: &str) -> Result<(), BackendError> {
        let mut git_commit = Command::new("git");

        git_commit.arg("commit");
        #[cfg(feature = "unstable-pre-commit")]
        git_commit.arg("--no-verify");
        git_commit
            .args(&self.extra_args)
            .args(["-em", commit_message]);

        tracing::info!(?git_commit, "calling git commit");

        let status = git_commit
            .status()
            .map_err(|os_error| BackendError::CannotRun {
                command: String::from("git commit"),
                os_error,
            })
            .log_err()?;

        tracing::debug!(?status);

        if !status.success() {
            Err(BackendError::ExecutionError {
                status_code: status.code(),
            })
            .log_err()?;
        }

        Ok(())
    }
}

/// A backend printing the message to the terminal.
pub struct PrintBackend;

impl Backend for PrintBackend {
    #[tracing::instrument(name = "print_backend", level = "trace", skip_all)]
    fn call(&self, commit_message: &str) -> Result<(), BackendError> {
        tracing::debug!("printing the commit message");
        println!("{commit_message}");
        Ok(())
    }
}
