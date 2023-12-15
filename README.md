# git-z

A Git extension to go beyond.

## Setup

### Installation with Nix

You can add `git-z` to your user profile by running:

    nix profile install github:ejpcmac/git-z

Alternatively, you can add `git-z` to your development environment by setting
up a `flake.nix` like this:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    git-z.url = "github:ejpcmac/git-z";
  };

  outputs = { flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = inputs.nixpkgs.legacyPackages.${system};
        git-z = inputs.git-z.packages.${system}.git-z;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            # Tools.
            git-z

            # Other dependencies.
          ];
        };
      }
    );
}
```

### Installation with Cargo

    cargo install git-z

## Usage

Run:

    git add <your modifications>
    git z commit

You can define a list of valid commit types, scopes, ticket prefixes and a
commit template by using a `git-z.toml` file at the root of your repository:

```toml
version = "0.1"
types = ["chore", "docs", "feat", "fix", "refactor", "revert", "style", "test"]
scopes = ["backend", "frontend"]
ticket_prefixes = [""]

template = """
{{ type }}{% if scope %}({{ scope }}){% endif %}{% if breaking_change %}!{% endif %}: {{ description }}

# Feel free to enter a longer description here.

Refs: {{ ticket }}

{% if breaking_change %}BREAKING CHANGE: {{ breaking_change }}{% endif %}
"""
```

For a more detailed description, please look at
[`commits.toml.sample`](./git-z.toml.sample).

## Building an installer

### Linux (Debian)

With Docker installed on your machine, you can run:

    $ ./build-deb.sh

You should then find a Debian package in `target/debian/`.

### Windows

With a Rust toolchain instalded on your machine, you can:

1. Install [WiX v3](https://wixtoolset.org/docs/wix3/).

2. Run:

        > cargo install cargo-wix
        > cargo wix --nocapture

You should find an installer in `target/wix/`.

## [Contributing](CONTRIBUTING.md)

Before contributing to this project, please read the
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright © 2023 Jean-Philippe Cugnet

This project is licensed under the [GNU General Public License 3.0](LICENSE).
