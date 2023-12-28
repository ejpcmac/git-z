# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2023-12-28

## Highlights

### New configuration format

The `git-z.toml` format has been updated to version 0.2 to provide a much better
configurability. It is now possible to:

* allow scopes to be arbitrary, instead of a list;
* allow the ticket reference to be optional, or even not asked for;

After updating git-z, to update your `git-z.toml` to the new configuration
format, you can run:

    git z update

`git-z 0.2.0` is still compatible with previous configurations, keeping their
semantics. However, **previous version of git-z cannot run with a configuration
version 0.2**.

### Working in any repository

The default configuration without a `git-z.toml` has been updated to be much
more sensible. You can then run `git z commit` without a configuration file and
still get something usable.

In addition, a new `git z init` command has been added to create a `git-z.toml`
in the current repository.

## Added

* [`git z init`] Add a command to create a `git-z.toml` in the current
    repository.
* [`git z update`] Add a command to update the configuration file without
    loosing comments and formatting.
* [Config] Add a `ticket.required` field: when set to `true`, the ticket is
    required as in previous versions. When set to `false`, the ticket is still
    asked for but optional.

## Changed

* **BREAKING** [Config] Update the configuration format to allow for more
    options.
* **BREAKING** [Config] Make the `ticket` key optional: when the ticket
    configuration is not present, no ticket will be asked for.
* **BREAKING** [Config] Allow scopes to be arbitrary.
* **BREAKING** [CLI] Print errors, warnings and hints on `stderr`.
* [Config] Make the default configuration more sensible. It now has much more
    built-in types, accepts an optional arbitrary scope, and no ticket.
* [CLI] Print better messages for many usage errors.
* [CLI] Do not print an error on cancelled / interrupted operations.
* [`git z commit`] Do not print an error on `git commit` failure.
* [`git z commit`] Check the commit template early and pretty-print any error
    mesasge.
* [`git z commit`] Consider `#` as a special ticket prefix: when matching in the
    branch name, `#` is omitted from the match so that branches like
    `feature/23-name` are valid, and `#` is then added to the matching ticket
    number. In this example it would extract `#23` as ticket number from the
    branch name.
* [`git z commit`] Enhance a bit the error message when failing to build a
    regex from the list of prefixes.
* [Cargo] Exclude unneeded files from the package.
* [Cargo] Update the dependencies.
* [Rust] Update from 1.74.0 to 1.74.1.

## Fixed

* [Config] Check the version before trying to parse using the latest
    configuration format. Previously, the configuration was parsed, then its
    `version` field was checked. This led to parsing errors instead of reporting
    an out of date configuration.

## [0.1.0] - 2023-12-01

### Added

* Initial version, featuring:
    * `git z commit`, a wizard that helps to buid a commit message by asking:
        * a type,
        * a optional scope,
        * a commit message (5-50 characters),
        * an optional breaking change,
        * a ticket number, pre-filled from the branch name;
    * a `git-z.toml` file at the root of the current Git repository allowing
        to configure:
        * the valid commit types,
        * the valid scopes,
        * the valid ticket prefixes,
        * the commit template.

[0.2.0]: https://github.com/ejpcmac/git-z/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ejpcmac/git-z/releases/tag/v0.1.0
