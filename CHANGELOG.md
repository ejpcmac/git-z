# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.4] - 2025-04-19

### Highlights

#### Adaptation to the size of the terminal

Before this version, selection prompts had a fixed size of 15 elements,
regardless of the size of the terminal. This was a bit annoying in small
terminals, as the rendering was likely to break.

Selection prompts now adapt their page size to the size of the terminal.

#### Experimental support for other VCS

This release introduces two new command line options to `git z commit`:

* `--topic <TOPIC>` allows to set the topic from which a ticket number is
    extracted, instead of the current Git branch,
* `--command <COMMAND>` enables to use a custom command instead of `git commit
    -em "$message"`.

The `<COMMAND>` argument value should contain a `$message` variable, which is
replaced by `git z commit` to the actual commit message.

These options can be used to integrate `git-z` with other VCS such as Jujutsu.
For instance, we can define a `jj-z-describe` shell function like this:

```sh
jj-z-describe() {
    # First, get the closest bookmarks on top of which we are working.
    local bookmarks="$(
        jj log --no-graph -r 'heads(::@ & bookmarks())' -T 'self.bookmarks()'
    )"

    # Then, call `git z commit` with the `--topic` and `--command` options. Note
    # that we are piping the message through `sed` to change any `#` Git comment
    # marker to `JJ:`.
    git z commit \
        --topic "$bookmarks" \
        --command "sh -c \"\
            echo -n '\$message' \
            | sed 's/^#\(.*\)/JJ:\1/' \
            | jj describe $@ --edit --stdin\" \
            "
}
```

Please note that this support is experimental, and that `git-z` still has to be
run from inside a Git repository. For using with Jujutsu, the repository must be
configured in colocated mode.

### Added

* [`git z commit`] Add a `--topic <TOPIC>` option to pass the name of the
    current topic via the command line ([#56]).
* [`git z commit`] Add a `--command <COMMAND>` option to pass a custom command
    to call instead of `git commit -em "$message"` ([#56]).

### Changed

* [`git z commit`] Adapt the size of the page in selection prompts to the size
    of the terminal ([#45]).
* [`git z commit`] Ensure there are no more than two consecutive newlines in
    commit messages.
* [Cargo] Update the dependencies.
* [Rust] Update from 1.84.1 to 1.86.0.

[#45]: https://github.com/ejpcmac/git-z/issues/45
[#56]: https://github.com/ejpcmac/git-z/issues/56

## [0.2.3] - 2025-02-19

### Changed

* [`git z commit`] Allow up to 60 characters in the description (was 50)
    ([#49]).
* [Cargo] Update the dependencies.
* [Rust] Update from 1.83.0 to 1.84.1.

### Fixed

* [CLI] Do not include the Git hash when running `git z -V` with a release
    binary built by the CI ([#38]).

[#38]: https://github.com/ejpcmac/git-z/issues/38
[#49]: https://github.com/ejpcmac/git-z/issues/49

## [0.2.2] - 2024-12-08

### Changed

* [CLI] Use `git z` instead of `git-z` in usage massages ([#39]).
* [Config] Improve the default description for the `refactor` commit type
    ([#41]).
* [Cargo] Update the dependencies.
* [Rust] Update from 1.81.0 to 1.83.0.

### Fixed

* [`git z commit`] Do not reuse an outdated commit message after the
    `pre-commit` hook has failed ([#47]).

[#39]: https://github.com/ejpcmac/git-z/issues/39
[#41]: https://github.com/ejpcmac/git-z/issues/41
[#47]: https://github.com/ejpcmac/git-z/issues/47

## [0.2.1] - 2024-09-25

### Added

* [`git z commit`] Ask whether to reuse the previous answers or commit message
    when the operation has been aborted or has failed ([#18]).
* [CLI] Add a new global `-v...` flag to control the log verbosity. By default,
    no logs are emitted ([#20]).
* [CLI] Add tracing logs in all layers ([#20]).
* *(unstable)* [`git z commit`] When compiling with the `unstable-pre-commit`
    feature enabled, run the `pre-commit` hook before the wizard if it exists. A
    new `-n|--no-verify` option is added to prevent the hook from running
    ([#11]).

### Changed

* [`git z commit`] Use fuzzy-finding when selecting types and scopes.
* [`git z commit`] Pass the extra arguments to `git commit` before `-em
    <message>`.
* [CLI] Provide more information in `git z --version`.
* [CLI] Use standard exit codes as defined in `sysexits.h` ([#22]).
* [Config] Enhance the descriptions for the default types ([#23], [#31]).
* [Config] Align error messages between the config and the updater.
* [Cargo] Update the dependencies.
* [Rust] Update from 1.74.1 to 1.81.0.
* *(unstable)* [`git z commit`] When compiling with the `unstable-pre-commit`
    feature enabled, passes `--no-verify` to `git commit`. This is to avoid
    running the `pre-commit` hook twice, but this also disables the `commit-msg`
    hook as an unwanted side-effect ([#11]).

### Removed

* [`git z update`] Remove support for updating from `0.2-dev.*` versions.
* [Config] Remove support for `0.2-dev.*` versions.

### Fixed

* [`git z commit`] Use the proper configuration name in hints.
* [Updater] Properly update the documentation for scopes. Previously, it was not
    updating the documentation above the `scopes` table, keeping the one form
    version 0.1.

[#18]: https://github.com/ejpcmac/git-z/issues/18
[#20]: https://github.com/ejpcmac/git-z/issues/20
[#22]: https://github.com/ejpcmac/git-z/issues/22
[#23]: https://github.com/ejpcmac/git-z/issues/23
[#31]: https://github.com/ejpcmac/git-z/issues/31

## [0.2.0] - 2023-12-28

### Highlights

#### New configuration format

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

#### Working in any repository

The default configuration without a `git-z.toml` has been updated to be much
more sensible. You can then run `git z commit` without a configuration file and
still get something usable.

In addition, a new `git z init` command has been added to create a `git-z.toml`
in the current repository.

### Added

* [`git z init`] Add a command to create a `git-z.toml` in the current
    repository ([#8]).
* [`git z update`] Add a command to update the configuration file without
    loosing comments and formatting.
* [Config] Add a `ticket.required` field: when set to `true`, the ticket is
    required as in previous versions. When set to `false`, the ticket is still
    asked for but optional ([#2]).

### Changed

* **BREAKING** [Config] Update the configuration format to allow for more
    options ([#4]).
* **BREAKING** [Config] Make the `ticket` key optional: when the ticket
    configuration is not present, no ticket will be asked for ([#2]).
* **BREAKING** [Config] Allow scopes to be arbitrary ([#3]).
* **BREAKING** [CLI] Print errors, warnings and hints on `stderr`.
* [Config] Make the default configuration more sensible. It now has much more
    built-in types, accepts an optional arbitrary scope, and no ticket ([#12]).
* [CLI] Print better messages for many usage errors ([#14]).
* [CLI] Do not print an error on cancelled / interrupted operations.
* [`git z commit`] Do not print an error on `git commit` failure.
* [`git z commit`] Check the commit template early and pretty-print any error
    message.
* [`git z commit`] Consider `#` as a special ticket prefix: when matching in the
    branch name, `#` is omitted from the match so that branches like
    `feature/23-name` are valid, and `#` is then added to the matching ticket
    number. In this example it would extract `#23` as ticket number from the
    branch name ([#7]).
* [`git z commit`] Enhance a bit the error message when failing to build a
    regex from the list of prefixes.
* [Cargo] Exclude unneeded files from the package.
* [Cargo] Update the dependencies.
* [Rust] Update from 1.74.0 to 1.74.1.

### Fixed

* [Config] Check the version before trying to parse using the latest
    configuration format. Previously, the configuration was parsed, then its
    `version` field was checked. This led to parsing errors instead of reporting
    an out of date configuration.

[#2]: https://github.com/ejpcmac/git-z/issues/2
[#3]: https://github.com/ejpcmac/git-z/issues/3
[#4]: https://github.com/ejpcmac/git-z/issues/4
[#7]: https://github.com/ejpcmac/git-z/issues/7
[#8]: https://github.com/ejpcmac/git-z/issues/8
[#11]: https://github.com/ejpcmac/git-z/issues/11
[#11]: https://github.com/ejpcmac/git-z/issues/11
[#12]: https://github.com/ejpcmac/git-z/issues/12
[#14]: https://github.com/ejpcmac/git-z/issues/14

## [0.1.0] - 2023-12-01

### Added

* Initial version, featuring:
    * `git z commit`, a wizard that helps to build a commit message by asking:
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

[0.2.4]: https://github.com/ejpcmac/git-z/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ejpcmac/git-z/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ejpcmac/git-z/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ejpcmac/git-z/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ejpcmac/git-z/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ejpcmac/git-z/releases/tag/v0.1.0
