{
  description = "A Git extension to go beyond.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devshell.flakeModule ];
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { self', system, ... }:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };
          rust-toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          ######################################################################
          ##                             Packages                             ##
          ######################################################################

          packages =
            let
              packageName = "git-z";

              naersk = pkgs.callPackage inputs.naersk {
                cargo = rust-toolchain;
                rustc = rust-toolchain;
              };

              mkPackage = { extraCargoBuildOptions ? [ ] }:
                naersk.buildPackage {
                  src = ./.;
                  cargoBuildOptions = opts: opts ++ extraCargoBuildOptions;
                  RUSTFLAGS = "-A missing_docs";
                  FLAKE_REVISION = self.shortRev or
                    (builtins.replaceStrings [ "dirty" ] [ "modified" ]
                      self.dirtyShortRev);

                  nativeBuildInputs = with pkgs; [ makeWrapper ];

                  postInstall = with pkgs; ''
                    wrapProgram $out/bin/${packageName} \
                      --prefix PATH : ${lib.makeBinPath [ git ]}
                  '';
                };
            in
            {
              default = self'.packages.${packageName};

              ${packageName} = mkPackage { };

              "${packageName}-unstable" = mkPackage {
                extraCargoBuildOptions = [ "--features unstable-pre-commit" ];
              };
            };

          ######################################################################
          ##                            Devshells                             ##
          ######################################################################

          devshells =
            let
              rustToolchain = version:
                if version == "stable" then
                  rust-toolchain
                else if version == "nightly" then
                  (pkgs.rust-bin.nightly."2025-10-20".minimal.override {
                    extensions = [ "llvm-tools" ];
                  })
                else throw "the Rust version must be `stable` or `nightly`";

              buildToolchain = version: with pkgs; [
                (rustToolchain version)
              ] ++ lib.optionals (!stdenv.isDarwin) [
                clang
              ];

              checkToolchain = with pkgs; [
                cargo-hack
                cargo-nextest
                committed
                eclint
                nixpkgs-fmt
                nodePackages.prettier
                reuse
                taplo
                typos
              ];

              nightlyCheckToolchain = with pkgs; [
                cargo-udeps
              ] ++ lib.optionals (!stdenv.isDarwin) [
                cargo-llvm-cov
              ];

              ideToolchain = with pkgs; [
                nixd
                rust-analyzer
              ];

              devTools = with pkgs; with self'.packages; [
                bacon
                cargo-bloat
                cargo-outdated
                git
                git-z
                gitflow
              ];

              testEnv = [
                {
                  name = "TEST_PATH";
                  eval = "$PRJ_ROOT/tests/fake_bin:${pkgs.bash}/bin";
                }
              ];

              devEnv = [
                {
                  name = "RUSTFLAGS";
                  value = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
                }
              ];

              ideEnv = [
                {
                  name = "NIX_PATH";
                  value = "nixpkgs=${inputs.nixpkgs}";
                }
                {
                  name = "TYPOS_LSP_PATH";
                  value = "${pkgs.typos-lsp}/bin/typos-lsp";
                }
              ];

              nightlyEnv = [
                {
                  name = "HAS_RUST_NIGHTLY";
                  value = "true";
                }
              ];
            in
            {
              default = {
                name = "git-z";

                motd = ''

                  {202}🔨 Welcome to the git-z devshell!{reset}
                '';

                packages =
                  buildToolchain "stable"
                  ++ checkToolchain
                  ++ ideToolchain
                  ++ devTools;

                env =
                  testEnv
                  ++ devEnv
                  ++ ideEnv;

                commands = [
                  {
                    name = "build-deb";
                    command = "cargo deb --target=x86_64-unknown-linux-musl";
                  }

                  # Pass-through commands to make some cargo extensions run with
                  # a different toolchain.
                  {
                    name = "cargo-deb";
                    command = "nix develop -L .#deb -c cargo $@";
                  }
                  {
                    name = "cargo-llvm-cov";
                    command = "nix develop -L .#rust-nightly -c cargo $@";
                  }
                  {
                    name = "cargo-udeps";
                    command = "nix develop -L .#rust-nightly -c cargo $@";
                  }
                  {
                    name = "coverage-report";
                    command = ''
                      nix develop -L .#rust-nightly -c \
                        cargo llvm-cov nextest --branch --open
                    '';
                  }
                  {
                    name = "live-coverage";
                    command = ''
                      nix develop -L .#rust-nightly -c bacon coverage
                    '';
                  }
                ];
              };

              # NOTE: Use the musl target to build a statically-linked binary.
              # We add the target in a specialised devshell to avoid cluttering
              # the toolchain defined in `rust-toolchain.toml` on all platforms.
              deb = {
                name = "cargo-deb";
                packages = with pkgs; [
                  (rust-toolchain.override {
                    targets = [ "x86_64-unknown-linux-musl" ];
                  })
                  clang
                  cargo-deb
                ];
              };

              # Devshell to run tools with a nightly toolchain.
              rust-nightly = {
                name = "Rust Nightly";

                packages =
                  buildToolchain "nightly"
                  ++ nightlyCheckToolchain;

                env =
                  nightlyEnv;
              };

              ci = {
                name = "git-z CI";

                packages =
                  buildToolchain "stable"
                  ++ checkToolchain;

                env =
                  testEnv;
              };

              ci-nightly = {
                name = "git-z CI (Rust Nightly)";

                packages =
                  buildToolchain "nightly"
                  ++ checkToolchain
                  ++ nightlyCheckToolchain;

                env =
                  testEnv
                  ++ nightlyEnv;
              };
            };
        };
    };
}
