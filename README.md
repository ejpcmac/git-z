# git-z

[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/ejpcmac/git-z/badge)](https://flakehub.com/flake/ejpcmac/git-z)
[![Crates.io](https://img.shields.io/crates/v/git-z)](https://crates.io/crates/git-z)
[![Crates.io License](https://img.shields.io/crates/l/git-z)](LICENSE)
[![Conventional
Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)
](https://conventionalcommits.org)

A Git extension to go beyond.

## Features

Currently available:

* A wizard to format commit messages according to [Conventional
    Commits](https://www.conventionalcommits.org/en/v1.0.0/). It is configurable
    with:
    * a list of valid commit types and their descriptions,
    * whether to ask for a scope,
    * if applicable, a list of valid scopes,
    * whether to ask for a ticket / issue reference,
    * automated ticket / issue reference information from the name of the
        branch,
    * a custom commit template.

On the roadmap:

* Support for other VCS such as [Jujutsu](https://github.com/jj-vcs/jj).
* Workflow commands _à la_ `git-flow`.
* A validator to ensure commit messages follow [Conventional
    Commits](https://www.conventionalcommits.org/en/v1.0.0/), optionally
    including a valid ticket reference.
* A wizard to create a branch—and optionally a worktree—from a GitHub / GitLab
    issue or Jira ticket.

## Setup

### Installation with Nix

If you are a **Nix** user on **Linux** or **macOS**, you can add `git-z` to your
user profile by running:

    nix profile install github:ejpcmac/git-z

Alternatively, you can add `git-z` to your development environment by setting
up a `flake.nix` like this:

<details>
<summary>Click to expand the example</summary>

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    git-z.url = "https://flakehub.com/f/ejpcmac/git-z/*";
  };

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      perSystem = { inputs', ... }:
        let
          pkgs = inputs'.nixpkgs.legacyPackages;
          git-z = inputs'.git-z.packages.git-z;
        in
        {
          devShells.default = pkgs.mkShell {
            buildInputs = [
              # Tools
              git-z

              # Other dependencies
            ];
          };
        };
    };
}
```

</details>

### Installation with Homebrew

If you are a **brew** user on **Linux** or **macOS**, you can install
`git-z` by running:

    brew install ejpcmac/repo/git-z

### Installation from the Debian package

If you are a **Debian** user—or of derivatives like **Ubuntu**—, you can install
`git-z` by running:

    curl -OL https://github.com/ejpcmac/git-z/releases/download/v0.2.4/git-z_0.2.4-1_amd64.deb
    sudo apt update
    sudo apt install ./git-z_0.2.4-1_amd64.deb

### Installation from the MSI package

If you are a **Windows** user, you can download an MSI package on the [the
release page](https://github.com/ejpcmac/git-z/releases/latest) and install it.
You may need to allow its execution by doing *Right Click > Properties*, then
checking the *Unblock* box in the security section at the bottom of the page.

### Installation from a pre-built binary

If you are a user of any other **Linux** distribution, **macOS** or **Windows**,
you can download a statically-linked executable on [the release
page](https://github.com/ejpcmac/git-z/releases/latest). Just rename it to
`git-z`—or `git-z.exe` on Windows—and put it somewhere in your `PATH`.

<details>
<summary>
  Click here for a detailed walkthrough for <strong>Linux</strong> and
  <strong>macOS</strong>.
</summary>

1. Download the executable from [the release
    page](https://github.com/ejpcmac/git-z/releases/latest).

2. Move the file to a folder of you choice, rename it and make it executable.

    ```sh
    # Create a local directory for manually installed executable files.
    mkdir -p ~/.local/bin

    # Move the executable to the right directory and renane it to `git-z`.
    cd downloads
    mv git-z-0.2.4-aarch64-apple-darwin ~/.local/bin/git-z

    # Don’t forget to make "git-z" executable.
    chmod +x ~/.local/bin/git-z
    ```

3. Ensure the folder in which you moved the file is referenced in your PATH.

    ```sh
    # Inside .bashrc, .zshrc or whatever shell config.
    PATH=$PATH:$HOME/.local/bin
    ```

4. On **macOS**, you may be facing a warning while trying to run git-z.

    Then, you have to remove the quarantine attribute from `git-z` using the
    `xattr` command in Terminal.

    ```sh
    # Remove the quarantine attribute from the `git-z` executable.
    xattr -d com.apple.quarantine ~/.local/bin/git-z
    ```

    There is also a graphical way to do it: you can refer to your system
    documentation.

</details>

### Installation with Cargo

If you are a **Rust programmer**, you can install `git-z` by running:

    cargo install git-z

## Usage

Run:

    git add <your modifications>
    git z commit

You can customise the behaviour of `git-z`:

* define the list of valid types with their description,
* choose whether to ask for a scope,
* define a list pre-defined valid scopes,
* ask or require a ticket / issue number.

To do this, initialise a configuration by running:

    git z init

Then, edit the `git-z.toml` at the root of your repository.

## Building an installer

### Linux (Debian)

From inside a Nix devshell, you can run:

    $ build-deb

You should then find a Debian package in
`target/x86_64-unknown-linux-musl/debian/`.

### Windows

With a Rust toolchain installed on your machine, you can:

1. Install [WiX v3](https://wixtoolset.org/docs/wix3/).

2. Run:

        > cargo install cargo-wix
        > cargo wix --package git-z --nocapture

You should find an installer in `target/wix/`.

## [Contributing](CONTRIBUTING.md)

Before contributing to this project, please read the
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright © 2023-2025 Jean-Philippe Cugnet

This project is licensed under the [GNU General Public License 3.0](LICENSE).
