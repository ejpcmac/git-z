# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
