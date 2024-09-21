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

//! Cargo xtasks for git-z.

use std::{
    env,
    process::{self, Command},
};

use colored::Colorize;
use xshell::{cmd, Shell};

struct Context {
    pub sh: Shell,
    pub checks: usize,
    pub errors: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            sh: Shell::new().unwrap(),
            checks: 0,
            errors: 0,
        }
    }
}

fn main() {
    let mut args = env::args();
    let _ = args.next();

    if let Some(command) = args.next().as_deref() {
        match command {
            "check" => check(args.next().as_deref()),
            _ => usage(),
        }
    } else {
        usage();
    }
}

fn usage() {
    let name = env::args().next().unwrap();
    eprintln!("usage: {name} <check>");
    process::exit(1);
}

////////////////////////////////////////////////////////////////////////////////
//                                  Commands                                  //
////////////////////////////////////////////////////////////////////////////////

fn check(subcommand: Option<&str>) {
    let mut ctx = Context::new();

    if let Some(check) = subcommand {
        match check {
            "commits" => check_commits(&mut ctx),
            "format" => check_format(&mut ctx),
            "unused-deps" => check_unused_deps(&mut ctx),
            "build" => build(&mut ctx),
            "test" => test(&mut ctx),
            _ => check_usage(),
        }
    } else {
        check_commits(&mut ctx);
        check_format(&mut ctx);
        check_unused_deps(&mut ctx);
        build(&mut ctx);
        test(&mut ctx);
    }

    check_result(&ctx);
}

fn check_usage() {
    let name = env::args().next().unwrap();
    eprintln!("usage: {name} check [commits|format|unused-deps|build|test]");
    process::exit(1);
}

//////////////////////////////////// Checks ////////////////////////////////////

fn check_commits(ctx: &mut Context) {
    let is_pull_request =
        env::var_os("IS_PULL_REQUEST").is_some_and(|val| val == "true");

    let commits = if is_pull_request {
        Some(String::from("HEAD~..HEAD^2"))
    } else {
        let branch = get_current_branch();

        if ["main", "develop"].contains(&branch.as_str()) {
            None
        } else {
            let merge_base = get_merge_base("origin/develop");
            Some(format!("{merge_base}..HEAD"))
        }
    };

    if let Some(commits) = commits {
        action!(
            ctx,
            step!(
                "Listing the commits to check",
                "git --no-pager log --pretty=format:'%C(yellow)%h%Creset %s' {commits}",
            ),
            step!(
                "Checking that commits are conventional",
                "committed {commits}",
            ),
        );
    }
}

fn check_format(ctx: &mut Context) {
    let editorconfig_excluded_files = [
        "**.lock",
        "**.rs",
        "**.toml",
        "LICENSE",
        "templates/*",
        "wix/gpl-3.0.rtf",
        "wix/main.wxs",
    ];

    let exclude = format!("{{{}}}", editorconfig_excluded_files.join(","));

    action!(
        ctx,
        "Checking for compliance with Editorconfig",
        "eclint -exclude {exclude}",
    );

    action!(
        ctx,
        "Checking that the Rust code is formatted",
        "cargo fmt --all --check",
    );

    action!(
        ctx,
        "Checking that the Nix code is formatted",
        "nixpkgs-fmt --check .",
    );

    action!(
        ctx,
        "Checking that TOML documents are formatted",
        "taplo fmt --check",
    );

    action!(ctx, "Checking for typos", "typos");
}

fn check_unused_deps(ctx: &mut Context) {
    action!(
        ctx,
        "Looking for unused dependencies",
        "nix develop -L .#udeps -c cargo hack udeps --workspace --all-targets --feature-powerset --keep-going",
    );
}

fn build(ctx: &mut Context) {
    action!(
        ctx,
        "Building all packages with all feature combinations",
        "cargo hack build --workspace --all-targets --feature-powerset --keep-going",
    );

    action!(
        ctx,
        "Checking for clippy warnings in all packages with all feature combinations",
        "cargo hack clippy --workspace --all-targets --feature-powerset --keep-going -- -D warnings",
    );
}

fn test(ctx: &mut Context) {
    action!(
        ctx,
        step!(
            "Building the tests for all packages with all feature combinations",
            "cargo hack nextest run --workspace --feature-powerset --keep-going --no-run",
        ),
        step!(
            "Running the tests for all packages with all feature combinations",
            "cargo hack nextest run --workspace --feature-powerset --keep-going",
        ),
    );
}

////////////////////////////////////////////////////////////////////////////////
//                                  Helpers                                   //
////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! action {
    ($ctx:ident, $name:literal, $command:literal $(,)?) => {{
        action!($ctx, step!($name, $command));
    }};

    ($ctx:ident, step!($name:literal, $command:literal $(,)?) $(,)?) => {{
        let result = step!($ctx, $name, $command);
        $ctx.checks += 1;
        let message = if result.is_ok() {
            "✅ PASSED".bold().green()
        } else {
            $ctx.errors += 1;
            "❌ FAILED".bold().red()
        };

        println!("{message}");
    }};

    (
        $ctx:ident,
        step!($name:literal, $command:literal $(,)?),
        $(step!($names:literal, $commands:literal $(,)?)),+
        $(,)?
    ) => {{
        let result = step!($ctx, $name, $command);
        if result.is_ok() {
            println!();
            action!($ctx, $(step!($names, $commands)),+);
        } else {
            $ctx.checks += 1;
            $ctx.errors += 1;
            let message = "❌ FAILED".bold().red();
            println!("{message}");
        }
    }};
}

#[macro_export]
macro_rules! step {
    ($ctx:ident, $name:literal, $command:literal $(,)?) => {{
        let _step = Step::new($name);
        cmd!($ctx.sh, $command).run()
    }};
}

struct Step;

impl Step {
    pub fn new(name: &'static str) -> Self {
        let message = if env::var_os("GITHUB_ACTIONS").is_some() {
            format!("::group::{name}")
        } else {
            format!("\n==> {name}...").bold().to_string()
        };

        println!("{message}");

        Self
    }
}

impl Drop for Step {
    fn drop(&mut self) {
        if env::var_os("GITHUB_ACTIONS").is_some() {
            println!("::endgroup::");
        }
    }
}

fn check_result(ctx: &Context) {
    let Context { checks, errors, .. } = ctx;

    let s = if *checks == 1 { "" } else { "s" };
    let have = |n| if n == 1 { "has" } else { "have" };

    if *errors == 0 {
        let have = have(*checks);
        let message = format!("✅ {checks}/{checks} check{s} {have} passed!")
            .bold()
            .green();
        println!("\n{message}");
    } else {
        let have = have(*errors);
        let message = format!("❌ {errors}/{checks} check{s} {have} failed!")
            .bold()
            .red();
        eprintln!("\n{message}");
        process::exit(1);
    }
}

fn get_current_branch() -> String {
    let git_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .unwrap();

    assert!(
        git_branch.status.success(),
        "Failed to run `git branch --show-current`"
    );

    String::from_utf8(git_branch.stdout)
        .unwrap()
        .trim()
        .to_owned()
}

fn get_merge_base(into: &str) -> String {
    let git_merge_base = Command::new("git")
        .args(["merge-base", into, "HEAD"])
        .output()
        .unwrap();

    assert!(
        git_merge_base.status.success(),
        "Failed to run `git merge-base origin/develop HEAD`"
    );

    String::from_utf8(git_merge_base.stdout)
        .unwrap()
        .trim()
        .to_owned()
}
