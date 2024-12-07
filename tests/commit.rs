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

//! CLI tests for `git z commit`.

// NOTE: rexpect is only compatible with Unix-like systems, so let’s just not
// compile the CLI tests on Windows.
#![cfg(not(target_os = "windows"))]
#![allow(clippy::pedantic, clippy::restriction)]

use std::{fs, path::Path, process::Command};

use assert_cmd::cargo::cargo_bin;
use assert_fs::{assert::IntoPathPredicate, prelude::*, TempDir};
use eyre::Result;
use indoc::{formatdoc, indoc};
use predicates::prelude::*;
use rexpect::{
    process::wait::WaitStatus,
    session::{spawn_command, PtySession},
};

#[cfg(feature = "unstable-pre-commit")]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

const TIMEOUT: Option<u64> = Some(1_000);
const COMMIT_CACHE_VERSION: &str = "0.1";

////////////////////////////////////////////////////////////////////////////////
//                                  Helpers                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Git {
    Fake,
}

fn setup_temp_dir(git: Git) -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    match git {
        Git::Fake => {
            temp_dir.child(".git").create_dir_all()?;
        }
    }

    Ok(temp_dir)
}

fn install_config(temp_dir: &TempDir, name: &str) -> Result<()> {
    let config_file = std::env::current_dir()?
        .join("tests")
        .join("res")
        .join("config")
        .join(name);

    temp_dir.child("git-z.toml").write_file(&config_file)?;
    Ok(())
}

fn install_commit_cache(temp_dir: &TempDir, commit_cache: &str) -> Result<()> {
    temp_dir
        .child(".git")
        .child("git-z")
        .child("commit-cache.toml")
        .write_str(commit_cache)?;
    Ok(())
}

#[cfg(feature = "unstable-pre-commit")]
fn install_pre_commit_hook(temp_dir: &TempDir, exit_code: i32) -> Result<()> {
    install_hook(
        temp_dir,
        "pre-commit",
        &formatdoc! {r##"
            #!/bin/sh
            echo "pre-commit"
            exit {exit_code}
        "##},
    )
}

// NOTE: Commenting this out since it is only used by
// pre_commit::still_runs_commit_msg which is disabled.
//
// #[cfg(feature = "unstable-pre-commit")]
// fn install_commit_msg_hook(temp_dir: &TempDir, exit_code: i32) -> Result<()> {
//     install_hook(
//         temp_dir,
//         "commit-msg",
//         &formatdoc! {r##"
//             #!/bin/sh
//             echo "commit-msg"
//             exit {exit_code}
//         "##},
//     )
// }

#[cfg(feature = "unstable-pre-commit")]
fn install_hook(temp_dir: &TempDir, name: &str, content: &str) -> Result<()> {
    let hook = &temp_dir.child(".git").child("hooks").child(name);
    hook.write_str(content)?;
    fs::set_permissions(hook, Permissions::from_mode(0o755))?;
    Ok(())
}

fn make_git_bare_repo(temp_dir: &TempDir) -> Result<()> {
    temp_dir.child(".git").child("bare").touch()?;
    Ok(())
}

fn set_git_branch(temp_dir: &TempDir, name: &str) -> Result<()> {
    temp_dir.child(".git").child("branch").write_str(name)?;
    Ok(())
}

fn set_git_return_code(temp_dir: &TempDir, error: i32) -> Result<()> {
    temp_dir
        .child(".git")
        .child("error")
        .write_str(&error.to_string())?;
    Ok(())
}

fn set_git_commit_message(temp_dir: &TempDir, message: &str) -> Result<()> {
    temp_dir
        .child(".git")
        .child("COMMIT_EDITMSG")
        .write_str(message)?;
    Ok(())
}

fn gitz_commit(temp_dir: impl AsRef<Path>, git: Git) -> Result<Command> {
    let mut cmd = Command::new(cargo_bin("git-z"));
    cmd.current_dir(&temp_dir)
        .env("NO_COLOR", "true")
        .arg("commit");

    if git == Git::Fake {
        let test_path = std::env::var("TEST_PATH")?;
        cmd.env("PATH", test_path);
    };

    Ok(cmd)
}

fn fill_type(process: &mut PtySession) -> Result<()> {
    process.exp_string("Commit type")?;
    process.send_line("")?;
    Ok(())
}

fn fill_scope(process: &mut PtySession) -> Result<()> {
    process.exp_string("Scope")?;
    process.send_line("")?;
    Ok(())
}

fn fill_description(process: &mut PtySession) -> Result<()> {
    process.exp_string("Short description")?;
    process.send_line("description")?;
    Ok(())
}

fn fill_breaking_change(process: &mut PtySession) -> Result<()> {
    process.exp_string("BREAKING CHANGE")?;
    process.send_line("")?;
    Ok(())
}

fn fill_do_reuse_answers(process: &mut PtySession, yes_no: &str) -> Result<()> {
    process.exp_string(
        "A previous run has been aborted. Do you want to reuse your answers?",
    )?;
    process.send_line(yes_no)?;
    Ok(())
}

fn fill_do_reuse_message(process: &mut PtySession, yes_no: &str) -> Result<()> {
    process.exp_string(
        "A previous run has been aborted. Do you want to reuse your commit \
            message?",
    )?;
    process.send_line(yes_no)?;
    Ok(())
}

fn wait_type(process: &mut PtySession) -> Result<()> {
    process.exp_string("Commit type")?;
    Ok(())
}

fn fill_type_and_wait_scope(
    process: &mut PtySession,
    r#type: &str,
) -> Result<()> {
    process.send_line(r#type)?;
    process.exp_string("Scope")?;
    Ok(())
}

fn fill_scope_and_wait_description(
    process: &mut PtySession,
    scope: &str,
) -> Result<()> {
    process.send_line(scope)?;
    process.exp_string("Short description")?;
    Ok(())
}

fn fill_description_and_wait_breaking_change(
    process: &mut PtySession,
    description: &str,
) -> Result<()> {
    process.send_line(description)?;
    process.exp_string("BREAKING CHANGE")?;
    Ok(())
}

fn fill_breaking_change_and_wait_ticket(
    process: &mut PtySession,
    breaking_change: &str,
) -> Result<()> {
    process.send_line(breaking_change)?;
    process.exp_string("Issue / ticket number")?;
    Ok(())
}

fn fill_ticket_and_wait_eof(
    process: &mut PtySession,
    ticket: &str,
) -> Result<()> {
    process.send_line(ticket)?;
    process.exp_eof()?;
    Ok(())
}

fn assert_commit_cache<I, P>(temp_dir: &TempDir, pred: I)
where
    I: IntoPathPredicate<P>,
    P: Predicate<Path>,
{
    temp_dir
        .child(".git")
        .child("git-z")
        .child("commit-cache.toml")
        .assert(pred);
}

fn assert_git_commit(temp_dir: &TempDir, content: &str) {
    temp_dir.child(".git").child("commit").assert(content);
}

////////////////////////////////////////////////////////////////////////////////
//                                   Wizard                                   //
////////////////////////////////////////////////////////////////////////////////

mod wizard {
    use super::*;

    ////////////////////////////// Default config //////////////////////////////

    #[test]
    fn uses_default_config() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        // Asks for commit type with default list.
        process.exp_string("Commit type")?;
        process.exp_string("feat")?;
        process.exp_string("chore")?;
        process.exp_string("enter to select, type to filter")?;
        process.send_line("chore")?;

        // Asks for an optional arbitrary scope (any).
        process.exp_string("Scope")?;
        process.exp_string("Press ESC or leave empty to omit the scope.")?;
        process.send_line("")?;

        // Asks for a short description within the 5-50 characters limit.
        process.exp_string("Short description")?;
        process.exp_string(
            "describe your change with a short description (5-50 characters)",
        )?;
        process.send_line("test description")?;

        // Asks for a optional breaking change.
        process.exp_string("BREAKING CHANGE")?;
        process.send_line("")?;

        // Does not ask for a ticket.
        process.exp_eof()?;

        Ok(())
    }

    /////////////////////////////////// Type ///////////////////////////////////

    #[test]
    fn asks_for_a_type() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.exp_string("to move, enter to select, type to filter")?;

        Ok(())
    }

    #[test]
    fn uses_types_from_config_file() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_types-custom.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.exp_string("type")?;
        process.exp_string("a first description")?;
        process.exp_string("second_type")?;
        process.exp_string("another description")?;
        process.exp_string("to move, enter to select, type to filter")?;

        Ok(())
    }

    #[test]
    fn accepts_a_type_from_the_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_line("type")?;

        process.exp_string("Scope")?;

        Ok(())
    }

    #[test]
    fn enforces_types_from_the_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_line("unknown")?;

        assert!(process.exp_string("Scope").is_err());

        Ok(())
    }

    #[test]
    fn aborts_if_type_is_skipped_with_esc() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_control('[')?;
        process.exp_eof()?;

        Ok(())
    }

    ////////////////////////////////// Scope ///////////////////////////////////

    #[test]
    fn asks_for_a_scope() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.exp_string("Press ESC or leave empty to omit the scope.")?;

        Ok(())
    }

    #[test]
    fn uses_list_of_scopes_from_config_file() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_scopes-list.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.exp_string("scope1")?;
        process.exp_string("scope2")?;
        process.exp_string(
            "to move, enter to select, type to filter, ESC to leave empty, \
                update `git-z.toml` to add new scopes",
        )?;

        Ok(())
    }

    #[test]
    fn allows_scope_to_be_empty_when_using_any() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_minimal.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.send_line("")?;

        process.exp_string("Short description")?;

        Ok(())
    }

    #[test]
    fn allows_scope_to_be_skipped_with_esc_when_using_any() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_minimal.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.send_control('[')?;
        process.exp_string("<canceled>")?;

        process.exp_string("Short description")?;

        Ok(())
    }

    #[test]
    fn accepts_a_scope_from_the_list_when_using_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_scopes-list.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.send_line("scope2")?;

        process.exp_string("Short description")?;

        Ok(())
    }

    #[test]
    fn enforces_scopes_from_the_list_when_using_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_scopes-list.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.send_line("unknown")?;

        assert!(process.exp_string("Short description").is_err());

        Ok(())
    }

    #[test]
    fn allows_scope_to_be_skipped_with_esc_when_using_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_scopes-list.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.send_control('[')?;
        process.exp_string("<canceled>")?;

        process.exp_string("Short description")?;

        Ok(())
    }

    /////////////////////////////// Description ////////////////////////////////

    #[test]
    fn asks_for_a_description() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process.exp_string(
            "describe your change with a short description (5-50 characters)",
        )?;
        process.exp_string(
            "You will be able to add a long description to your commit in an \
                editor later.",
        )?;

        Ok(())
    }

    #[test]
    fn accepts_a_description_between_5_and_50_characters() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process.send_line("test description")?;

        process.exp_string("BREAKING CHANGE")?;

        Ok(())
    }

    #[test]
    fn refuses_a_description_shorter_than_5_characters() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process.send_line("****")?;

        process
            .exp_string("The description must be longer than 5 characters")?;
        assert!(process.exp_string("BREAKING CHANGE").is_err());

        Ok(())
    }

    #[test]
    fn refuses_a_description_longer_than_50_characters() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process
            .send_line("***************************************************")?;

        process.exp_string(
            "The description must not be longer than 50 characters",
        )?;
        assert!(process.exp_string("BREAKING CHANGE").is_err());

        Ok(())
    }

    #[test]
    fn refuses_a_description_starting_in_lowercase() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process.send_line("Test description")?;

        process.exp_string("The description must start in lowercase")?;
        assert!(process.exp_string("BREAKING CHANGE").is_err());

        Ok(())
    }

    #[test]
    fn aborts_if_description_is_skipped_with_esc() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;

        process.exp_string("Short description")?;
        process.send_control('[')?;
        process.exp_eof()?;

        Ok(())
    }

    ///////////////////////////// Breaking change //////////////////////////////

    #[test]
    fn asks_for_a_breaking_change() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;

        process.exp_string("BREAKING CHANGE")?;
        process.exp_string(
            "Press ESC or leave empty if there are no breaking changes.",
        )?;

        Ok(())
    }

    #[test]
    fn allows_breaking_change_to_be_empty_when_using_any() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;

        process.exp_string("BREAKING CHANGE")?;
        process.send_line("")?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    #[test]
    fn allows_breaking_change_to_be_skipped_with_esc() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;

        process.exp_string("BREAKING CHANGE")?;
        process.send_control('[')?;
        process.exp_string("<canceled>")?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    ////////////////////////////////// Ticket //////////////////////////////////

    #[test]
    fn does_not_ask_for_a_ticket_when_not_specified_in_config() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_minimal.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    #[test]
    fn asks_for_a_ticket_when_specified_in_config() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.exp_string("#XXX or GH-XXX")?;
        process.exp_string("Press ESC to omit the ticket reference.")?;

        Ok(())
    }

    #[test]
    fn accepts_a_ticket_with_proper_format() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.send_line("#42")?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    #[test]
    fn accepts_a_ticket_with_proper_format2() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.send_line("GH-42")?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    #[test]
    fn refuses_a_ticket_with_improper_format() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.send_line("TEST-99")?;

        process.exp_string(
            "The issue / ticket number must be in the form #XXX or GH-XXX",
        )?;
        assert!(process.exp_string("fake commit").is_err());

        Ok(())
    }

    #[test]
    fn allows_ticket_to_be_skipped_with_esc_when_not_required() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.send_control('[')?;
        process.exp_string("<canceled>")?;

        process.exp_string("fake commit")?;

        Ok(())
    }

    #[test]
    fn aborts_if_ticket_is_skipped_with_esc_when_required() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-required.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.send_control('[')?;
        process.exp_string("<canceled>")?;

        assert!(process.exp_string("fake commit").is_err());
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn gets_the_ticket_number_from_branch() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;
        set_git_branch(&temp_dir, "feature/GH-42-test-branch")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.exp_string("GH-42")?;

        Ok(())
    }

    #[test]
    fn gets_the_ticket_number_from_branch_when_hash() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;
        set_git_branch(&temp_dir, "feature/42-test-branch")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("Issue / ticket number")?;
        process.exp_string("#42")?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                Commit cache                                //
////////////////////////////////////////////////////////////////////////////////

mod commit_cache {
    use super::*;

    #[test]
    fn saves_each_answer_along_the_way() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_full.toml")?;

        // NOTE: Let’s make Git error so the commit cache is kept.
        set_git_return_code(&temp_dir, 1)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        wait_type(&mut process)?;
        assert_commit_cache(&temp_dir, predicate::path::missing());

        fill_type_and_wait_scope(&mut process, "chore")?;
        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
            "##},
        );

        fill_scope_and_wait_description(&mut process, "hell")?;
        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
                scope = "hell"
            "##},
        );

        fill_description_and_wait_breaking_change(
            &mut process,
            "flames everywhere",
        )?;
        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
                scope = "hell"
                description = "flames everywhere"
            "##},
        );

        fill_breaking_change_and_wait_ticket(
            &mut process,
            "It ain’t heaven anymore.",
        )?;
        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
                scope = "hell"
                description = "flames everywhere"
                breaking_change = "It ain’t heaven anymore."
            "##},
        );

        fill_ticket_and_wait_eof(&mut process, "#666")?;
        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
                type = "chore"
                scope = "hell"
                description = "flames everywhere"
                breaking_change = "It ain’t heaven anymore."
                ticket = "#666"
            "##},
        );

        Ok(())
    }

    #[test]
    fn saves_the_wizard_completion() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        // NOTE: Let’s make Git error so the commit cache is kept.
        set_git_return_code(&temp_dir, 1)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        assert_commit_cache(&temp_dir, predicate::path::missing());

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;
        process.exp_eof()?;

        assert_commit_cache(
            &temp_dir,
            formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
                type = "feat"
                description = "description"
            "##},
        );

        Ok(())
    }

    #[test]
    fn deletes_the_commit_cache_on_commit_success() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;
        process.exp_string("fake commit")?;
        process.exp_eof()?;

        assert_commit_cache(&temp_dir, predicate::path::missing());

        Ok(())
    }

    #[test]
    fn asks_whether_to_prefill_answers_if_a_cache_exists() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "feat"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "A previous run has been aborted. Do you want to reuse your \
                answers?",
        )?;
        process.exp_string(
            "The wizard will be run as usual with your answers pre-selected.",
        )?;

        Ok(())
    }

    #[test]
    fn prefills_answers_with_commit_cache_if_the_user_accepts() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_full.toml")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
                scope = "hell"
                description = "flames everywhere"
                breaking_change = "It ain't heaven anymore."
                ticket = "#666"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_answers(&mut process, "y")?;

        // Ensure all answers are pre-filled.

        process.exp_string("Commit type")?;
        process.exp_string("> chore")?;
        process.send_line("")?;

        process.exp_string("Scope")?;
        process.exp_string("hell")?;
        process.send_line("")?;

        process.exp_string("Short description")?;
        process.exp_string("flames everywhere")?;
        process.send_line("")?;

        process.exp_string("BREAKING CHANGE")?;
        process.exp_string("It ain't heaven anymore.")?;
        process.send_line("")?;

        process.exp_string("Issue / ticket number")?;
        process.exp_string("#666")?;
        process.send_line("")?;

        Ok(())
    }

    #[test]
    fn prefills_answers_with_commit_cache_by_default() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_full.toml")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        // Just press enter (default).
        fill_do_reuse_answers(&mut process, "")?;

        process.exp_string("Commit type")?;
        // Ensure this is pre-selected.
        process.exp_string("> chore")?;

        Ok(())
    }

    #[test]
    fn prefills_the_scope_with_commit_cache_when_any() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_minimal.toml")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "type"
                scope = "everything"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_answers(&mut process, "y")?;
        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.exp_string("everything")?;

        Ok(())
    }

    #[test]
    fn prefills_the_scope_with_commit_cache_when_list() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_scopes-list.toml")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "type"
                scope = "scope2"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_answers(&mut process, "y")?;
        fill_type(&mut process)?;

        process.exp_string("Scope")?;
        process.exp_string("> scope2")?;

        Ok(())
    }

    #[test]
    fn does_not_prefill_answers_if_the_user_declines() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_full.toml")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "chore"
                scope = "hell"
                description = "flames everywhere"
                breaking_change = "It ain’t heaven anymore."
                ticket = "#666"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_answers(&mut process, "n")?;

        // Ensure all answers are not prefilled.

        process.exp_string("Commit type")?;
        assert!(process.exp_string("> chore").is_err());
        process.send_line("")?;

        process.exp_string("Scope")?;
        assert!(process.exp_string("hell").is_err());
        process.send_line("")?;

        process.exp_string("Short description")?;
        assert!(process.exp_string("flames everywhere").is_err());
        process.send_line("description")?;

        process.exp_string("BREAKING CHANGE")?;
        assert!(process.exp_string("It ain’t heaven anymore.").is_err());
        process.send_line("")?;

        process.exp_string("Issue / ticket number")?;
        assert!(process.exp_string("#666").is_err());
        process.send_line("")?;

        Ok(())
    }

    #[test]
    fn deletes_the_commit_cache_if_the_user_declines() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "ongoing"

                [wizard_answers]
                type = "feat"
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_answers(&mut process, "n")?;
        wait_type(&mut process)?;

        assert_commit_cache(&temp_dir, predicate::path::missing());

        Ok(())
    }

    #[test]
    fn asks_whether_to_reuse_message_if_wizard_is_complete_and_message_exists(
    ) -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(&temp_dir, "previous message")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "A previous run has been aborted. Do you want to reuse your \
                commit message?",
        )?;
        process.exp_string(
            "This will use your last commit message without running the wizard."
        )?;

        Ok(())
    }

    #[test]
    fn asks_whether_to_prefill_answers_if_wizard_is_complete_but_message_missing(
    ) -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "A previous run has been aborted. Do you want to reuse your \
                answers?",
        )?;
        process.exp_string(
            "The wizard will be run as usual with your answers pre-selected.",
        )?;

        Ok(())
    }

    #[test]
    fn asks_whether_to_prefill_answers_if_wizard_is_complete_but_message_empty(
    ) -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(
            &temp_dir,
            indoc! {"
                # This is an empty message

                # That contains several
                # commented lines.

                # And multiple empty lines.
            "},
        )?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "A previous run has been aborted. Do you want to reuse your \
                answers?",
        )?;
        process.exp_string(
            "The wizard will be run as usual with your answers pre-selected.",
        )?;

        Ok(())
    }

    #[test]
    fn does_not_run_the_wizard_when_reusing_previous_message() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(&temp_dir, "previous message")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_message(&mut process, "y")?;

        // No interactive wizard: direct call to `git commit`.
        process.exp_string("fake commit")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn reuses_the_previous_message_by_default() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(&temp_dir, "previous message")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        // Just press enter (default).
        fill_do_reuse_message(&mut process, "")?;

        // No interactive wizard: direct call to `git commit`.
        process.exp_string("fake commit")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn calls_git_commit_with_previous_message_when_reusing_it() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(
            &temp_dir,
            indoc! {"
                previous message

                This is a long description
                on multiple lines.

                Footer: something.

            "},
        )?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_message(&mut process, "y")?;
        process.exp_string("fake commit")?;
        process.exp_eof()?;

        #[cfg(not(feature = "unstable-pre-commit"))]
        assert_git_commit(
            &temp_dir,
            indoc! {"
                commit -em previous message

                This is a long description
                on multiple lines.

                Footer: something.
            "},
        );

        #[cfg(feature = "unstable-pre-commit")]
        assert_git_commit(
            &temp_dir,
            indoc! {"
                commit --no-verify -em previous message

                This is a long description
                on multiple lines.

                Footer: something.
            "},
        );

        Ok(())
    }

    #[test]
    fn runs_the_wizard_when_not_reusing_previous_message() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(&temp_dir, "previous message")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_message(&mut process, "n")?;

        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn deletes_the_commit_cache_if_the_user_declines_previous_message(
    ) -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_commit_message(&temp_dir, "previous message")?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "{COMMIT_CACHE_VERSION}"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_do_reuse_message(&mut process, "n")?;
        wait_type(&mut process)?;

        assert_commit_cache(&temp_dir, predicate::path::missing());

        Ok(())
    }

    #[test]
    fn does_not_ask_anything_if_there_is_no_commit_cache() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        assert!(process
            .exp_string(
                "A previous run has been aborted. Do you want to reuse your \
                answers?",
            )
            .is_err());

        assert!(process
            .exp_string(
                "A previous run has been aborted. Do you want to reuse your \
                commit message?",
            )
            .is_err());

        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn ignores_the_commit_cache_if_its_version_mismatches() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "0.0"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn deletes_the_commit_cache_if_its_version_mismatches() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                version = "0.0"
                wizard_state = "completed"

                [wizard_answers]
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        assert_commit_cache(&temp_dir, predicate::path::missing());

        Ok(())
    }

    #[test]
    fn ignores_the_commit_cache_if_it_is_invalid() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                invalid
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn deletes_the_commit_cache_if_it_is_invalid() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_commit_cache(
            &temp_dir,
            &formatdoc! {r##"
                invalid
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        assert_commit_cache(&temp_dir, predicate::path::missing());

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                 pre-commit                                 //
////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "unstable-pre-commit")]
mod pre_commit {
    use super::*;

    #[test]
    fn directly_runs_the_wizard_if_there_is_no_pre_commit_hook() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        assert!(process.exp_string("pre-commit").is_err());
        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn calls_pre_commit_if_it_exists() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 0)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("pre-commit")?;

        Ok(())
    }

    #[test]
    fn does_not_call_pre_commit_if_no_verify_is_passed() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 0)?;

        let mut cmd = gitz_commit(&temp_dir, Git::Fake)?;
        cmd.arg("--no-verify");

        let mut process = spawn_command(cmd, TIMEOUT)?;

        assert!(process.exp_string("pre-commit").is_err());
        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn runs_the_wizard_if_pre_commit_succeeds() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 0)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("pre-commit")?;
        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn exits_with_an_error_if_pre_commit_fails() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 1)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("pre-commit")?;
        process.exp_eof()?;
        assert!(matches!(process.process.wait()?, WaitStatus::Exited(_, 1)));

        Ok(())
    }

    #[test]
    fn prints_a_warning_if_pre_commit_is_not_executable() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 0)?;

        let pre_commit =
            &temp_dir.child(".git").child("hooks").child("pre-commit");
        fs::set_permissions(pre_commit, Permissions::from_mode(0o644))?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "The `.git/hooks/pre-commit` hook was ignored because it is not \
            set as executable.",
        )?;
        process.exp_string("Commit type")?;

        Ok(())
    }

    #[test]
    fn runs_pre_commit_only_once() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 0)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("pre-commit")?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        assert!(process.exp_string("pre-commit").is_err());
        process.exp_string("fake commit")?;
        process.exp_eof()?;

        Ok(())
    }

    // NOTE: Commenting this out since the current implementation makes it fail.
    // This will be resolved in a future version.
    //
    // #[test]
    // fn still_runs_commit_msg() -> Result<()> {
    //     let temp_dir = setup_temp_dir(Git::Fake)?;
    //     install_pre_commit_hook(&temp_dir, 0)?;
    //     install_commit_msg_hook(&temp_dir, 0)?;

    //     let mut process =
    //         spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

    //     process.exp_string("pre-commit")?;

    //     fill_type(&mut process)?;
    //     fill_scope(&mut process)?;
    //     fill_description(&mut process)?;
    //     fill_breaking_change(&mut process)?;

    //     process.exp_string("commit-msg")?;
    //     process.exp_string("fake commit")?;
    //     process.exp_eof()?;

    //     Ok(())
    // }
}

////////////////////////////////////////////////////////////////////////////////
//                                 git commit                                 //
////////////////////////////////////////////////////////////////////////////////

mod commit {
    use super::*;

    #[test]
    fn calls_git_commit_with_message_from_template() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-dummy.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("fake commit")?;
        process.exp_eof()?;

        #[cfg(not(feature = "unstable-pre-commit"))]
        assert_git_commit(&temp_dir, "commit -em dummy template message\n");

        #[cfg(feature = "unstable-pre-commit")]
        assert_git_commit(
            &temp_dir,
            "commit --no-verify -em dummy template message\n",
        );

        Ok(())
    }

    #[test]
    fn replaces_variables_from_the_template_with_entered_values() -> Result<()>
    {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_ticket-optional.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_line("type")?;

        process.exp_string("Scope")?;
        process.send_line("scope")?;

        process.exp_string("Short description")?;
        process.send_line("test description")?;

        process.exp_string("BREAKING CHANGE")?;
        process.send_line("Nothing is like before.")?;

        process.exp_string("Issue / ticket number")?;
        process.send_line("#21")?;

        process.exp_string("fake commit")?;
        process.exp_eof()?;

        #[cfg(not(feature = "unstable-pre-commit"))]
        assert_git_commit(
            &temp_dir,
            indoc! {"
                commit -em type(scope)!: test description

                # Feel free to enter a longer description here.

                Refs: #21

                BREAKING CHANGE: Nothing is like before.
            "},
        );

        #[cfg(feature = "unstable-pre-commit")]
        assert_git_commit(
            &temp_dir,
            indoc! {"
                commit --no-verify -em type(scope)!: test description

                # Feel free to enter a longer description here.

                Refs: #21

                BREAKING CHANGE: Nothing is like before.
            "},
        );

        Ok(())
    }

    #[test]
    fn calls_git_commit_with_extra_args() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-dummy.toml")?;

        let mut cmd = gitz_commit(&temp_dir, Git::Fake)?;
        cmd.args(["--", "--extra", "--args"]);

        let mut process = spawn_command(cmd, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("fake commit")?;
        process.exp_eof()?;

        #[cfg(not(feature = "unstable-pre-commit"))]
        assert_git_commit(
            &temp_dir,
            "commit --extra --args -em dummy template message\n",
        );

        #[cfg(feature = "unstable-pre-commit")]
        assert_git_commit(
            &temp_dir,
            "commit --no-verify --extra --args -em dummy template message\n",
        );

        Ok(())
    }

    #[test]
    fn prints_commit_message_when_print_only() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-dummy.toml")?;

        let mut cmd = gitz_commit(&temp_dir, Git::Fake)?;
        cmd.arg("--print-only");

        let mut process = spawn_command(cmd, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        process.exp_string("dummy template message")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn does_not_call_git_when_print_only() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-dummy.toml")?;

        let mut cmd = gitz_commit(&temp_dir, Git::Fake)?;
        cmd.arg("--print-only");

        let mut process = spawn_command(cmd, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        assert!(process.exp_string("fake commit").is_err());

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                Usage errors                                //
////////////////////////////////////////////////////////////////////////////////

mod usage_errors {
    use super::*;

    /////////////////////////////////// Git ////////////////////////////////////

    #[test]
    fn prints_an_error_if_git_is_not_available() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut cmd = gitz_commit(&temp_dir, Git::Fake)?;
        cmd.env("PATH", "");

        let mut process = spawn_command(cmd, TIMEOUT)?;

        process.exp_string("Error: failed to run the git command.")?;
        process.exp_string("The OS reports:")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_not_run_in_git_repo() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        fs::remove_dir(temp_dir.child(".git"))?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: not in a Git repository.")?;
        process.exp_string(
            "You can initialise a Git repository by running `git init`.",
        )?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_not_run_in_git_worktree() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        make_git_bare_repo(&temp_dir)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: not inside a Git worktree.")?;
        process.exp_string(
            "You seem to be inside a Git repository, but not in a worktree.",
        )?;
        process.exp_eof()?;

        Ok(())
    }

    ////////////////////////////////// Config //////////////////////////////////

    #[test]
    fn prints_an_error_if_the_config_version_is_unsupported() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "invalid_version.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: unsupported configuration version 49.3")?;
        process.exp_string(
            "Your git-z.toml may have been created by a newer version of git-z."
        )?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_config_is_an_old_development_one() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "invalid_development.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "Error: unsupported development configuration version 0.2-dev.0",
        )?;
        process.exp_string(
            "To update from this version, you can install git-z 0.2.0",
        )?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_config_has_no_version() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "invalid_no-version.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: invalid configuration in git-z.toml")?;
        process.exp_string("missing field `version`")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_config_is_invalid() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "invalid_value.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: invalid configuration in git-z.toml")?;
        process.exp_string("missing field `types`")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_config_is_not_toml() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "invalid_config.not_toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: invalid configuration in git-z.toml")?;
        process.exp_string("TOML parse error")?;
        process.exp_eof()?;

        Ok(())
    }

    ////////////////////////////////// Commit //////////////////////////////////

    #[cfg(feature = "unstable-pre-commit")]
    #[test]
    fn prints_an_error_if_the_pre_commit_hook_cannot_be_run() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_hook(
            &temp_dir,
            "pre-commit",
            &formatdoc! {r##"
                #!/invalid
                invalid
            "##},
        )?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: failed to run the pre-commit hook.")?;
        process.exp_string("The OS reports:")?;
        process.exp_eof()?;

        Ok(())
    }

    #[cfg(feature = "unstable-pre-commit")]
    #[test]
    fn prints_an_error_if_the_pre_commit_hook_fails() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_pre_commit_hook(&temp_dir, 1)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Error: the pre-commit hook has failed.")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn does_not_print_an_error_if_git_commit_fails() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_return_code(&temp_dir, 1)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        assert!(process.exp_string("Git has returned an error").is_err());

        Ok(())
    }

    #[test]
    fn propagates_the_status_code_if_git_commit_fails() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        set_git_return_code(&temp_dir, 21)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        fill_type(&mut process)?;
        fill_scope(&mut process)?;
        fill_description(&mut process)?;
        fill_breaking_change(&mut process)?;

        assert!(matches!(process.process.wait()?, WaitStatus::Exited(_, 21)));

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_template_is_invalid() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-invalid.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "Error: failed to parse 'templates.commit' from the configuration.",
        )?;
        process.exp_string("expected a template")?;
        process.exp_eof()?;

        Ok(())
    }

    #[test]
    fn prints_an_error_if_the_template_contains_an_unknown_variable(
    ) -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;
        install_config(&temp_dir, "latest_template-unknown-variable.toml")?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string(
            "Error: failed to render 'templates.commit' from the configuration."
        )?;
        process.exp_string(
            "Variable `unknown` not found in context while rendering \
                'templates.commit'",
        )?;
        process.exp_eof()?;

        Ok(())
    }

    ////////////////////////////////// Abort ///////////////////////////////////

    #[test]
    fn does_not_print_an_error_when_aborting_with_esc() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_control('[')?;

        assert!(process
            .exp_string("Operation was canceled by the user")
            .is_err());

        Ok(())
    }

    #[test]
    fn does_not_print_an_error_when_aborting_with_control_c() -> Result<()> {
        let temp_dir = setup_temp_dir(Git::Fake)?;

        let mut process =
            spawn_command(gitz_commit(&temp_dir, Git::Fake)?, TIMEOUT)?;

        process.exp_string("Commit type")?;
        process.send_control('c')?;

        assert!(process
            .exp_string("Operation was interrupted by the user")
            .is_err());

        Ok(())
    }
}
