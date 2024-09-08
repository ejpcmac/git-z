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

//! The Command Line Interface for git-z.

mod commit;
mod helpers;
mod init;
mod update;

use std::error::Error as _;

use clap::{ArgAction, Parser, Subcommand};
use eyre::{Report, Result};
use inquire::InquireError;
use tracing_subscriber::fmt::format::FmtSpan;

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

/// The long version information.
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nrevision: ",
    env!("REVISION"),
    "\nfeatures: ",
    env!("FEATURES"),
    "\ntarget: ",
    env!("TARGET"),
    "\nprofile: ",
    env!("PROFILE"),
    "\nbuilt by: ",
    env!("BUILT_BY"),
);

/// A Git extension to go beyond.
#[derive(Debug, Parser)]
#[command(
    author,
    version = env!("VERSION_WITH_GIT"),
    long_version = LONG_VERSION,
)]
pub struct GitZ {
    /// The command to run.
    #[command(subcommand)]
    command: GitZCommand,
    /// The verbosity level.
    #[arg(short = 'v', action = ArgAction::Count, global = true)]
    verbosity: u8,
}

/// The subcommands of `git-z`.
#[derive(Debug, Subcommand)]
pub enum GitZCommand {
    /// Initialises the configuration.
    Init(Init),
    /// Runs the commit wizard.
    Commit(Commit),
    /// Updates the configuration.
    Update(Update),
}

/// A command.
trait Command {
    /// Runs the command.
    fn run(&self) -> Result<()>;
}

impl GitZ {
    /// Runs git-z.
    pub fn run() -> Result<()> {
        let args = Self::parse();
        setup_tracing(args.verbosity);

        let result = match args.command {
            GitZCommand::Init(init) => init.run(),
            GitZCommand::Commit(commit) => commit.run(),
            GitZCommand::Update(update) => update.run(),
        };

        match result {
            Err(error) => handle_errors(error),
            Ok(()) => Ok(()),
        }
    }
}

/// Configures the tracing subscriber given the verbosity.
fn setup_tracing(verbosity: u8) {
    tracing_subscriber::fmt()
        .with_env_filter(env_filter(verbosity))
        .with_span_events(span_events(verbosity))
        .init();
}

/// Returns the trace filter to apply given the verbosity.
fn env_filter(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "off",
        1 => "git_z=info",
        2 => "git_z=debug",
        3_u8..=u8::MAX => "git_z=trace",
    }
}

/// Returns the span events to enable given the verbosity.
fn span_events(verbosity: u8) -> FmtSpan {
    match verbosity {
        0..=3 => FmtSpan::NONE,
        4..=u8::MAX => FmtSpan::ACTIVE,
    }
}

/// How to handle the error.
enum ErrorHandling {
    /// Return the report.
    Return(Report),
    /// Exit the program with the given status code.
    Exit(i32),
}

/// Handles typical usage errors to enhance their output.
fn handle_errors(error: Report) -> Result<()> {
    let handling = if let Some(error) = error.downcast_ref::<NotInGitWorktree>()
    {
        handle_not_in_git_worktree(error)
    } else if let Some(config::LoadError::InvalidConfig(error)) =
        error.downcast_ref::<config::LoadError>()
    {
        handle_from_toml_error(error)
    } else if let Some(updater::LoadError::InvalidConfig(error)) =
        error.downcast_ref::<updater::LoadError>()
    {
        handle_from_toml_error(error)
    } else if let Some(error) = error.downcast_ref::<InitError>() {
        handle_init_error(error)
    } else if let Some(error) = error.downcast_ref::<CommitError>() {
        handle_commit_error(error)
    } else if let Some(error) = error.downcast_ref::<UpdateError>() {
        handle_update_error(error)
    } else if let Some(InquireError::OperationCanceled) =
        error.downcast_ref::<InquireError>()
    {
        ErrorHandling::Exit(exitcode::TEMPFAIL)
    } else if let Some(InquireError::OperationInterrupted) =
        error.downcast_ref::<InquireError>()
    {
        ErrorHandling::Exit(exitcode::TEMPFAIL)
    } else {
        ErrorHandling::Return(error)
    };

    match handling {
        ErrorHandling::Return(error) => Err(error),
        ErrorHandling::Exit(code) => {
            #[allow(clippy::exit)]
            std::process::exit(code);
        }
    }
}

/// Prints proper error messages when running `git-z` outside of a Git worktree.
fn handle_not_in_git_worktree(error: &NotInGitWorktree) -> ErrorHandling {
    match error {
        NotInGitWorktree::CannotRunGit(os_error) => {
            error!("{error}.");
            hint!("The OS reports: {os_error}.");
            ErrorHandling::Exit(exitcode::UNAVAILABLE)
        }
        NotInGitWorktree::NotInRepo => {
            error!("{error}.");
            hint!("You can initialise a Git repository by running `git init`.");
            ErrorHandling::Exit(exitcode::USAGE)
        }
        NotInGitWorktree::NotInWorktree => {
            error!("{error}.");
            hint!("You seem to be inside a Git repository, but not in a worktree.");
            ErrorHandling::Exit(exitcode::USAGE)
        }
    }
}

/// Prints proper error messages for configuration loading errors.
fn handle_from_toml_error(error: &FromTomlError) -> ErrorHandling {
    match error {
        FromTomlError::UnsupportedVersion { .. } => {
            error!("{error}.");
            hint!("Your {CONFIG_FILE_NAME} may have been created by a newer version of git-z.");
        }
        FromTomlError::UnsupportedDevelopmentVersion {
            gitz_version, ..
        } => {
            error!("{error}.");
            hint! {"
                Your {CONFIG_FILE_NAME} has been created by a development version of git-z.
                However, configurations produced by a development version are only
                supported by the immediately following release.

                To update from this version, you can install git-z {gitz_version}
                run `git z update`, then update to the latest version and run
                `git z update` again.\
            "};
        }
        FromTomlError::ParseError(parse_error) => {
            error!("Invalid configuration in {CONFIG_FILE_NAME}.");
            hint!("\n{parse_error}");
        }
    }

    ErrorHandling::Exit(exitcode::CONFIG)
}

/// Prints proper error messages for `git z init` usage errors.
fn handle_init_error(error: &InitError) -> ErrorHandling {
    match error {
        InitError::ExistingConfig => {
            error!("{error}.");
            hint!("You can force the command by running `git z init -f`.");
        }
    }

    ErrorHandling::Exit(exitcode::CANTCREAT)
}

/// Prints proper error messages for `git z commit` usage errors.
fn handle_commit_error(error: &CommitError) -> ErrorHandling {
    match error {
        #[cfg(feature = "unstable-pre-commit")]
        CommitError::CannotRunPreCommit(os_error) => {
            error!("{error}.");
            hint!("The OS reports: {os_error}.");
            ErrorHandling::Exit(exitcode::UNAVAILABLE)
        }
        #[cfg(feature = "unstable-pre-commit")]
        CommitError::PreCommitFailed => {
            error!("{error}.");
            // NOTE: Use 1 as exit code to maintain the same behaviour as Git.
            ErrorHandling::Exit(1)
        }
        CommitError::Git { status_code } => {
            ErrorHandling::Exit(status_code.unwrap_or(1_i32))
        }
        CommitError::Template(tera_error) => {
            error!("{tera_error} from the configuration.");

            if let Some(parse_error) = tera_error.source() {
                hint!("\n{parse_error}\n");
            }

            ErrorHandling::Exit(exitcode::CONFIG)
        }
    }
}

/// Prints proper error messages for `git z update` usage errors.
fn handle_update_error(error: &UpdateError) -> ErrorHandling {
    match error {
        UpdateError::UnsupportedVersion { .. } => {
            error!("{error}.");
            hint!("Your {CONFIG_FILE_NAME} may have been created by a newer version of git-z.");
        }
        UpdateError::UnsupportedDevelopmentVersion { gitz_version, .. } => {
            error!("{error}.");
            hint! {"
                `git z update` can update a configuration from any previous release.
                However, configurations produced by a development version can only be
                updated by the immediately following release.

                To update from this version, you can install git-z {gitz_version},
                run `git z update`, then update to the latest version and run
                `git z update` again.\
            "};
        }
    }

    ErrorHandling::Exit(exitcode::CONFIG)
}
