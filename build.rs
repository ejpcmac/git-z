//! The build script for git-z.

use std::{env, fs, io, process::Command};

use serde::{Deserialize, Serialize};

fn main() {
    define_version_with_git();
    define_revision();
    define_features();
    define_target();
    define_profile();
    define_built_by();
}

/// Defines a variable containing the version with the Git revision.
///
/// `VERSION_WITH_GIT` contains at least the cargo version, even when Git is not
/// available. When Git is available, the current Git revision and dirty state
/// is added to the version as a tag, except in the following cases:
///
/// * the build is done from a clean worktree checked out at a tag matching
///   *exactly* the cargo version prefixed by `v`,
/// * the build is done from a cargo checkout (via `cargo install --git`) and
///   the version does not contain `-dev`.
///
/// If Git is not available, but the `FLAKE_REVISION` environment variable is
/// defined, it is used instead to provide the revision from the Nix flake, for
/// development versions only.
///
/// If neither Git nor the `FLAKE_REVISION` are available, but a
/// `.cargo_vcs_info.json` is present, it is used to provide the revision, for
/// development versions only.
///
/// For instance:
///
/// * Cargo version 1.0.0 on tag v1.0.0, clean state => `1.0.0`
/// * Cargo version 1.0.0 on tag v1.0.0, dirty state =>
///   `1.0.0+abcd1234-modified`
/// * Cargo version 1.1.0-dev on any commit, clean state => `1.1.0-dev+abcd1234`
fn define_version_with_git() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let version = version_with_features(cargo_version);
    let version = version_with_revision(&version);
    println!("cargo:rustc-env=VERSION_WITH_GIT={version}");
}

/// Returns the version with feature flags.
fn version_with_features(cargo_version: &str) -> String {
    let features = features().join("+");

    if features.is_empty() {
        String::from(cargo_version)
    } else {
        format!("{cargo_version}+{features}")
    }
}

/// Returns the list of enabled features.
fn features() -> Vec<&'static str> {
    if env::var("CARGO_FEATURE_UNSTABLE_PRE_COMMIT").is_ok() {
        vec!["unstable-pre-commit"]
    } else {
        vec![]
    }
}

/// Returns the version from cargo with a revision.
fn version_with_revision(cargo_version: &str) -> String {
    if let Some(revision) = maybe_revision(cargo_version) {
        format!("{cargo_version}+{revision}")
    } else {
        String::from(cargo_version)
    }
}

/// Gets the revision from the Git or the flake if applicable.
fn maybe_revision(cargo_version: &str) -> Option<String> {
    maybe_revision_from_git(cargo_version)
        .ok()
        .flatten()
        .or_else(|| maybe_revision_from_flake(cargo_version))
        .or_else(|| maybe_revision_from_cargo_vcs_info(cargo_version))
}

/// Gets the revision from Git if applicable.
fn maybe_revision_from_git(cargo_version: &str) -> io::Result<Option<String>> {
    if git_describe()?.is_some_and(|s| s == format!("v{cargo_version}"))
        || is_cargo_checkout()? && !is_dev_version(cargo_version)
    {
        Ok(None)
    } else {
        Ok(git_revision_and_state()?)
    }
}

/// Returns the result of `git describe --always --dirty=-modified`.
#[expect(
    clippy::missing_panics_doc,
    reason = "The unwrap in the function cannot actually panic on modern systems."
)]
fn git_describe() -> io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["describe", "--always", "--dirty=-modified"])
        .output()?;

    #[expect(clippy::unwrap_used, reason = "Non-UTF-8 outputs are obsolete.")]
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).unwrap().trim().to_owned()))
}

/// Returns the current Git revision and its dirtiness.
fn git_revision_and_state() -> io::Result<Option<String>> {
    git_revision()?
        .map(|revision| {
            if git_is_dirty()? && !is_cargo_checkout()? {
                Ok(format!("{revision}-modified"))
            } else {
                Ok(revision)
            }
        })
        .transpose()
}

/// Returns the current Git revision.
#[expect(
    clippy::missing_panics_doc,
    reason = "The unwrap in the function cannot actually panic on modern systems."
)]
fn git_revision() -> io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;

    #[expect(clippy::unwrap_used, reason = "Non-UTF-8 outputs are obsolete.")]
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).unwrap().trim().to_owned()))
}

/// Returns whether the current Git worktree is dirty.
fn git_is_dirty() -> io::Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Returns whether the current worktree is a checkout from cargo.
fn is_cargo_checkout() -> io::Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    Ok(output.status.success() && output.stdout == b"?? .cargo-ok\n")
}

/// Gets the revision from the flake if applicable.
fn maybe_revision_from_flake(cargo_version: &str) -> Option<String> {
    if is_dev_version(cargo_version) {
        env::var("FLAKE_REVISION").ok()
    } else {
        None
    }
}

/// Gets the revision from the `.cargo_vcs_info.json` if applicable.
fn maybe_revision_from_cargo_vcs_info(cargo_version: &str) -> Option<String> {
    if is_dev_version(cargo_version) {
        revision_from_cargo_vcs_info().ok()
    } else {
        None
    }
}

/// Gets the revision from the `.cargo_vcs_info.json`.
fn revision_from_cargo_vcs_info() -> io::Result<String> {
    /// Contents of the `.cargo_vcs_info.json`.
    #[derive(Serialize, Deserialize)]
    struct CargoVcsInfo {
        /// The Git information.
        git: GitInfo,
    }

    /// Git information.
    #[derive(Serialize, Deserialize)]
    struct GitInfo {
        /// The commit hash.
        sha1: String,
        /// Whether the worktree was dirty.
        dirty: Option<bool>,
    }

    let vcs_info: CargoVcsInfo =
        serde_json::from_str(&fs::read_to_string(".cargo_vcs_info.json")?)?;

    let revision = vcs_info.git.sha1.chars().take(8).collect();
    let revision = if vcs_info.git.dirty == Some(true) {
        format!("{revision}-modified")
    } else {
        revision
    };

    Ok(revision)
}

/// Returns whether the current version is a development version.
fn is_dev_version(cargo_version: &str) -> bool {
    cargo_version.contains("-dev")
}

/// Defines a variable containing the Git revision.
fn define_revision() {
    let revision = revision();
    println!("cargo:rustc-env=REVISION={revision}");
}

/// Gets the revision from Git or the flake.
fn revision() -> String {
    git_revision_and_state()
        .ok()
        .flatten()
        .or_else(|| env::var("FLAKE_REVISION").ok())
        .or_else(|| revision_from_cargo_vcs_info().ok())
        .unwrap_or_default()
}

/// Defines a variable containing the list of enabled features.
fn define_features() {
    let features = features().join(", ");
    println!("cargo:rustc-env=FEATURES={features}");
}

/// Passes the `TARGET` variable to the build.
#[expect(
    clippy::missing_panics_doc,
    reason = "The unwrap in the function cannot actually panic."
)]
fn define_target() {
    #[expect(clippy::unwrap_used, reason = "TARGET is defined by cargo")]
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-env=TARGET={target}");
}

/// Passes the `PROFILE` variable to the build.
#[expect(
    clippy::missing_panics_doc,
    reason = "The unwrap in the function cannot actually panic."
)]
fn define_profile() {
    #[expect(clippy::unwrap_used, reason = "PROFILE is defined by cargo")]
    let profile = env::var("PROFILE").unwrap();
    println!("cargo:rustc-env=PROFILE={profile}");
}

/// Defines a variable containing the name of the build tool.
fn define_built_by() {
    let built_by = built_by();
    println!("cargo:rustc-env=BUILT_BY={built_by}");
}

/// Returns the name of the build tool.
fn built_by() -> &'static str {
    if env::var("FLAKE_REVISION").is_ok() {
        "nix"
    } else {
        "cargo"
    }
}
