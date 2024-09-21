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
                  RUSTFLAGS = "-Amissing_docs";
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
              buildToolchain = with pkgs; [
                rust-toolchain
              ] ++ lib.optionals (!stdenv.isDarwin) [
                clang
              ];

              checkToolchain = with pkgs; [
                cargo-hack
                cargo-nextest
                committed
                eclint
                nixpkgs-fmt
                taplo
                typos
                yamlfmt
              ];

              ideToolchain = with pkgs; [
                nixd
                rust-analyzer
              ];

              developmentTools = with pkgs; with self'.packages; [
                cargo-bloat
                cargo-outdated
                cargo-watch
                git
                git-z
                gitAndTools.gitflow
              ];

              testEnv = [
                {
                  name = "TEST_PATH";
                  eval = "$PRJ_ROOT/tests/fake_bin:${pkgs.bash}/bin";
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
            in
            {
              default = {
                name = "git-z";

                motd = ''

                {202}🔨 Welcome to the git-z devshell!{reset}
              '';

                packages =
                  buildToolchain
                  ++ checkToolchain
                  ++ ideToolchain
                  ++ developmentTools;

                env =
                  testEnv
                  ++ ideEnv;
              };

              ci = {
                name = "git-z CI";

                packages =
                  buildToolchain
                  ++ checkToolchain;

                env =
                  testEnv;
              };

              # NOTE: Use the musl target to build a statically-linked binary.
              # We only add the target in a specialised devshell to avoid
              # cluttering the toolchain defined in `rust-toolchain.toml` on all
              # platforms.
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

              # NOTE: cargo-udeps needs Rust nightly to run.
              udeps = {
                name = "cargo-udeps";
                packages = with pkgs; [
                  rust-bin.nightly."2024-08-27".minimal
                  clang
                  cargo-hack
                  cargo-udeps
                ];
              };
            };
        };
    };
}
