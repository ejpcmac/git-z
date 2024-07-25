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

// NOTE: rexpect is only compatible with Unix-like systems, so let’s just not
// compile the CLI tests on Windows.
#![cfg(not(target_os = "windows"))]
#![allow(clippy::pedantic, clippy::restriction)]

use std::{fs, path::Path, process::Command};

use assert_cmd::cargo::cargo_bin;
use assert_fs::{prelude::*, TempDir};
use eyre::Result;
use rexpect::{
    process::wait::WaitStatus,
    session::{spawn_command, PtySession},
};

const TIMEOUT: Option<u64> = Some(1_000);

////////////////////////////////////////////////////////////////////////////////
//                                  Helpers                                   //
////////////////////////////////////////////////////////////////////////////////

fn setup_temp_dir() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    temp_dir.child(".git").create_dir_all()?;
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

fn set_git_branch(temp_dir: &TempDir, name: &str) -> Result<()> {
    temp_dir.child(".git").child("branch").write_str(name)?;
    Ok(())
}

fn gitz_commit(temp_dir: impl AsRef<Path>) -> Result<Command> {
    let test_path = std::env::var("TEST_PATH")?;

    let mut cmd = Command::new(cargo_bin("git-z"));
    cmd.current_dir(&temp_dir)
        .env("PATH", test_path)
        .arg("commit");

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

////////////////////////////////////////////////////////////////////////////////
//                                   Wizard                                   //
////////////////////////////////////////////////////////////////////////////////

//////////////////////////////// Default config ////////////////////////////////

#[test]
fn test_commit_wizard_uses_default_config() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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

///////////////////////////////////// Type /////////////////////////////////////

#[test]
fn test_commit_wizard_asks_for_a_type() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.exp_string("to move, enter to select, type to filter")?;

    Ok(())
}

#[test]
fn test_commit_wizard_uses_types_from_config_file() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_types-custom.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.exp_string("type")?;
    process.exp_string("a first description")?;
    process.exp_string("second_type")?;
    process.exp_string("another description")?;
    process.exp_string("to move, enter to select, type to filter")?;

    Ok(())
}

#[test]
fn test_commit_wizard_accepts_a_type_from_the_list() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.send_line("type")?;

    process.exp_string("Scope")?;

    Ok(())
}

#[test]
fn test_commit_wizard_enforces_types_from_the_list() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.send_line("unknown")?;

    assert!(process.exp_string("Scope").is_err());

    Ok(())
}

#[test]
fn test_commit_wizard_aborts_if_type_is_skipped_with_esc() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.send_control('[')?;
    process.exp_eof()?;

    Ok(())
}

//////////////////////////////////// Scope /////////////////////////////////////

#[test]
fn test_commit_wizard_asks_for_a_scope() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.exp_string("Press ESC or leave empty to omit the scope.")?;

    Ok(())
}

#[test]
fn test_commit_wizard_uses_list_of_scopes_from_config_file() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_scopes-list.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.exp_string("scope1")?;
    process.exp_string("scope2")?;
    process.exp_string(&format!(
        "{}, {}, {}",
        "to move, enter to select, type to filter",
        "ESC to leave empty",
        "update `git-z.toml` to add new scopes"
    ))?;

    Ok(())
}

#[test]
fn test_commit_wizard_allows_scope_to_be_empty_when_using_any() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_minimal.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.send_line("")?;

    process.exp_string("Short description")?;

    Ok(())
}

#[test]
fn test_commit_wizard_allows_scope_to_be_skipped_with_esc_when_using_any(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_minimal.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.send_control('[')?;
    process.exp_string("<canceled>")?;

    process.exp_string("Short description")?;

    Ok(())
}

#[test]
fn test_commit_wizard_accepts_a_scope_from_the_list_when_using_list(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_scopes-list.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.send_line("scope2")?;

    process.exp_string("Short description")?;

    Ok(())
}

#[test]
fn test_commit_wizard_enforces_scopes_from_the_list_when_using_list(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_scopes-list.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.send_line("unknown")?;

    assert!(process.exp_string("Short description").is_err());

    Ok(())
}

#[test]
fn test_commit_wizard_allows_scope_to_be_skipped_with_esc_when_using_list(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_scopes-list.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;

    process.exp_string("Scope")?;
    process.send_control('[')?;
    process.exp_string("<canceled>")?;

    process.exp_string("Short description")?;

    Ok(())
}

///////////////////////////////// Description //////////////////////////////////

#[test]
fn test_commit_wizard_asks_for_a_description() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.exp_string(
        "describe your change with a short description (5-50 characters)",
    )?;
    process.exp_string("You will be able to add a long description to your commit in an editor later.")?;

    Ok(())
}

#[test]
fn test_commit_wizard_accepts_a_description_between_5_and_50_characters(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.send_line("test description")?;

    process.exp_string("BREAKING CHANGE")?;

    Ok(())
}

#[test]
fn test_commit_wizard_refuses_a_description_shorter_than_5_characters(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.send_line("****")?;

    process.exp_string("The description must be longer than 5 characters")?;
    assert!(process.exp_string("BREAKING CHANGE").is_err());

    Ok(())
}

#[test]
fn test_commit_wizard_refuses_a_description_longer_than_50_characters(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.send_line("***************************************************")?;

    process
        .exp_string("The description must not be longer than 50 characters")?;
    assert!(process.exp_string("BREAKING CHANGE").is_err());

    Ok(())
}

#[test]
fn test_commit_wizard_refuses_a_description_starting_in_lowercase() -> Result<()>
{
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.send_line("Test description")?;

    process.exp_string("The description must start in lowercase")?;
    assert!(process.exp_string("BREAKING CHANGE").is_err());

    Ok(())
}

#[test]
fn test_commit_wizard_aborts_if_description_is_skipped_with_esc() -> Result<()>
{
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;

    process.exp_string("Short description")?;
    process.send_control('[')?;
    process.exp_eof()?;

    Ok(())
}

/////////////////////////////// Breaking change ////////////////////////////////

#[test]
fn test_commit_wizard_asks_for_a_breaking_change() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_allows_breaking_change_to_be_empty_when_using_any(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;

    process.exp_string("BREAKING CHANGE")?;
    process.send_line("")?;

    process.exp_string("fake commit")?;

    Ok(())
}

#[test]
fn test_commit_wizard_allows_breaking_change_to_be_skipped_with_esc(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;

    process.exp_string("BREAKING CHANGE")?;
    process.send_control('[')?;
    process.exp_string("<canceled>")?;

    process.exp_string("fake commit")?;

    Ok(())
}

//////////////////////////////////// Ticket ////////////////////////////////////

#[test]
fn test_commit_wizard_does_not_ask_for_a_ticket_when_not_specified_in_config(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_minimal.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    process.exp_string("fake commit")?;

    Ok(())
}

#[test]
fn test_commit_wizard_asks_for_a_ticket_when_specified_in_config() -> Result<()>
{
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_accepts_a_ticket_with_proper_format() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_accepts_a_ticket_with_proper_format2() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_refuses_a_ticket_with_improper_format() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_allows_ticket_to_be_skipped_with_esc_when_not_required(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_aborts_if_ticket_is_skipped_with_esc_when_required(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-required.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_wizard_gets_the_ticket_number_from_branch() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;
    set_git_branch(&temp_dir, "feature/GH-42-test-branch")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    process.exp_string("Issue / ticket number")?;
    process.exp_string("GH-42")?;

    Ok(())
}

#[test]
fn test_commit_wizard_gets_the_ticket_number_from_branch_when_hash(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;
    set_git_branch(&temp_dir, "feature/42-test-branch")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    process.exp_string("Issue / ticket number")?;
    process.exp_string("#42")?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
//                                 git commit                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_commit_calls_git_commit_with_message_from_template() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-dummy.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    process.exp_string("fake commit")?;
    process.exp_eof()?;

    temp_dir
        .child(".git")
        .child("commit")
        .assert("commit -em dummy template message\n\n");

    Ok(())
}

#[test]
fn test_commit_replaces_variables_from_the_template_with_entered_values(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_ticket-optional.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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

    temp_dir.child(".git").child("commit").assert(
        "commit -em type(scope)!: test description\n\
        \n\
        # Feel free to enter a longer description here.\n\
        \n\
        Refs: #21\n\
        \n\
        BREAKING CHANGE: Nothing is like before.\n\n",
    );

    Ok(())
}

#[test]
fn test_commit_calls_git_commit_with_extra_args() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-dummy.toml")?;

    let mut cmd = gitz_commit(&temp_dir)?;
    cmd.args(["--", "--extra", "--args"]);

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    process.exp_string("fake commit")?;
    process.exp_eof()?;

    temp_dir
        .child(".git")
        .child("commit")
        .assert("commit --extra --args -em dummy template message\n\n");

    Ok(())
}

#[test]
fn test_commit_prints_commit_message_when_print_only() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-dummy.toml")?;

    let mut cmd = gitz_commit(&temp_dir)?;
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
fn test_commit_does_not_call_git_when_print_only() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-dummy.toml")?;

    let mut cmd = gitz_commit(&temp_dir)?;
    cmd.arg("--print-only");

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    assert!(process.exp_string("fake commit").is_err());

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
//                                Usage errors                                //
////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////// Git //////////////////////////////////////

#[test]
fn test_commit_prints_an_error_if_git_is_not_available() -> Result<()> {
    let temp_dir = setup_temp_dir()?;

    let mut cmd = gitz_commit(&temp_dir)?;
    cmd.env("PATH", "");

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: failed to run the git command.")?;
    process.exp_string("The OS reports:")?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_not_run_in_git_repo() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    fs::remove_dir(temp_dir.child(".git"))?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: not in a Git repository.")?;
    process.exp_string(
        "You can initialise a Git repository by running `git init`.",
    )?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_not_run_in_git_worktree() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    temp_dir.child(".git").child("bare").touch()?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: not inside a Git worktree.")?;
    process.exp_string(
        "You seem to be inside a Git repository, but not in a worktree.",
    )?;
    process.exp_eof()?;

    Ok(())
}

//////////////////////////////////// Config ////////////////////////////////////

#[test]
fn test_commit_prints_an_error_if_the_config_version_is_unsupported(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "invalid_version.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: unsupported configuration version 49.3")?;
    process.exp_string(
        "Your git-z.toml may have been created by a newer version of git-z.",
    )?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_the_config_is_an_old_development_one(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "invalid_development.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

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
fn test_commit_prints_an_error_if_the_config_has_no_version() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "invalid_no-version.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: invalid configuration in git-z.toml")?;
    process.exp_string("missing field `version`")?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_the_config_is_invalid() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "invalid_value.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: invalid configuration in git-z.toml")?;
    process.exp_string("missing field `types`")?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_the_config_is_not_toml() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "invalid_config.not_toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Error: invalid configuration in git-z.toml")?;
    process.exp_string("TOML parse error")?;
    process.exp_eof()?;

    Ok(())
}

//////////////////////////////////// Commit ////////////////////////////////////

#[test]
fn test_commit_does_not_print_an_error_if_git_commit_fails() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    temp_dir.child(".git").child("error").touch()?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    assert!(process.exp_string("Git has returned an error").is_err());

    Ok(())
}

#[test]
fn test_commit_propagates_the_status_code_if_git_commit_fails() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    temp_dir.child(".git").child("error").write_str("21")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    fill_type(&mut process)?;
    fill_scope(&mut process)?;
    fill_description(&mut process)?;
    fill_breaking_change(&mut process)?;

    assert!(matches!(process.process.wait()?, WaitStatus::Exited(_, 21)));

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_the_template_is_invalid() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-invalid.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string(
        "Error: failed to parse 'templates.commit' from the configuration.",
    )?;
    process.exp_string("expected a template")?;
    process.exp_eof()?;

    Ok(())
}

#[test]
fn test_commit_prints_an_error_if_the_template_contains_an_unknown_variable(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    install_config(&temp_dir, "latest_template-unknown-variable.toml")?;

    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string(
        "Error: failed to render 'templates.commit' from the configuration.",
    )?;
    process.exp_string("Variable `unknown` not found in context while rendering 'templates.commit'")?;
    process.exp_eof()?;

    Ok(())
}

//////////////////////////////////// Abort /////////////////////////////////////

#[test]
fn test_commit_does_not_print_an_error_when_aborting_with_esc() -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.send_control('[')?;

    assert!(process
        .exp_string("Operation was canceled by the user")
        .is_err());

    Ok(())
}

#[test]
fn test_commit_does_not_print_an_error_when_aborting_with_control_c(
) -> Result<()> {
    let temp_dir = setup_temp_dir()?;
    let cmd = gitz_commit(&temp_dir)?;

    let mut process = spawn_command(cmd, TIMEOUT)?;

    process.exp_string("Commit type")?;
    process.send_control('c')?;

    assert!(process
        .exp_string("Operation was interrupted by the user")
        .is_err());

    Ok(())
}
