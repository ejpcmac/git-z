# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Added

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

[Unreleased]: https://github.com/ejpcmac/git-z/compare/main...develop
[0.1.0]: https://github.com/ejpcmac/git-z/releases/tag/v0.1.0
