# Contributing to git-z

git-z is written in [Rust](https://www.rust-lang.org/).

For branching management, this project uses
[git-flow](https://github.com/petervanderdoes/gitflow-avh). The `main` branch is
reserved for releases: the development process occurs on `develop` and feature
branches. **Please never commit to `main`.**

## Setup

### Local repository

1. Fork the repository.

2. Clone your fork to a local repository:

        git clone https://github.com/you/git-z.git
        cd git-z

3. Add the main repository as a remote:

        git remote add upstream https://github.com/ejpcmac/git-z.git

4. Checkout `develop`:

        git checkout develop

### Building the project

1. Build the project:

        cd git-z
        cargo build

2. Run the tests:

        cargo test

All the tests should pass.

## Workflow

To make a change, please use this workflow:

1. Checkout `develop` and apply the last upstream changes (use rebase, not
    merge!):

        git checkout develop
        git fetch --all --prune
        git rebase upstream/develop

2. For a tiny patch, create a new branch with an explicit name:

        git checkout -b <my_branch>

    Alternatively, if you are working on a feature which would need more work,
    you can create a feature branch with `git-flow`:

        git flow feature start <my_feature>

    *Note: always open an issue and ask before starting a big feature, to avoid
    it not beeing merged and your time lost.*

3. Work on your feature (don’t forget to write tests):

        # Some work
        git z commit
        # Some work
        git z commit
        ...

4. When your feature is ready, feel free to use
    [interactive rebase](https://help.github.com/articles/about-git-rebase/) so
    your history looks clean and is easy to follow. Then, apply the last
    upstream changes on `develop` to prepare integration:

        git checkout develop
        git fetch --all --prune
        git rebase upstream/develop

5. If there were commits on `develop` since the beginning of your feature
    branch, integrate them by **rebasing** if your branch has few commits, or
    merging if you had a long-lived branch:

        git checkout <my_feature_branch>
        git rebase develop

    *Note: the only case you should merge is when you are working on a big
    feature. If it is the case, we should have discussed this before as stated
    above.*

6. Run the tests to ensure there is no regression and all works as expected:

        cargo test

7. If it’s all good, open a pull request to merge your branch into the `develop`
    branch on the main repository.

## Coding style

Please format your code with `rustfmt`.

All contributed code must be documented. In general, take your inspiration from
the existing code.

## Commit style

Please name your commits using [Conventional
Commits](https://www.conventionalcommits.org/en/v1.0.0/) and using the types and
scopes defined in `git-z.toml`. You can use `git z commit` to help you prepare
the commit message.
