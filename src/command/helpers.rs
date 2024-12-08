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

//! Helpers for writing CLIs.

use std::{io, process::Command};

use eyre::Result;
use thiserror::Error;

use crate::{
    config::{Config, CONFIG_FILE_NAME, VERSION},
    hint,
    tracing::LogResult as _,
    warning,
};

/// Errors that can occur when not inside a Git worktree.
#[derive(Debug, Error)]
pub enum NotInGitWorktree {
    /// Git cannot be run.
    #[error("Failed to run the git command")]
    CannotRunGit(#[source] io::Error),
    /// The command is not run from inside a Git repository.
    #[error("Not in a Git repository")]
    NotInRepo,
    /// The command is not run from inside a Git worktree.
    #[error("Not inside a Git worktree")]
    NotInWorktree,
}

/// Ensures the command is run from a Git worktree.
#[tracing::instrument(level = "trace")]
pub fn ensure_in_git_worktree() -> Result<(), NotInGitWorktree> {
    let is_inside_work_tree = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(NotInGitWorktree::CannotRunGit)
        .log_err()?;

    if !is_inside_work_tree.status.success() {
        return Err(NotInGitWorktree::NotInRepo).log_err();
    }

    if is_inside_work_tree.stdout == b"true\n" {
        Ok(())
    } else {
        Err(NotInGitWorktree::NotInWorktree).log_err()
    }
}

/// Loads the configuration.
#[tracing::instrument(level = "trace")]
pub fn load_config() -> Result<Config> {
    let config = Config::load()?;

    if config.version != VERSION {
        warning!("The configuration in {CONFIG_FILE_NAME} is out of date.");
        hint!("You can update it by running `git z update`.");
    }

    Ok(config)
}

/// Prints a success.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*).green().bold();
        println!("{message}");
    }};
}

/// Prints a warning.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;

        let log_message = $crate::helpers::uncapitalise(&format!($($arg)*));
        let log_message = log_message.trim_end_matches(".");
        tracing::warn!("{log_message}");

        let message = indoc::formatdoc!($($arg)*).yellow().bold();
        eprintln!("{message}");
    }};
}

/// Prints an error.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*);
        let message = $crate::helpers::uncapitalise(&message);
        let message = format!("Error: {message}").red().bold();
        eprintln!("{message}");
    }};
}

/// Prints a hint.
#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*).blue();
        eprintln!("{message}");
    }};
}
