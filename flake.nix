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

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devshell.flakeModule ];
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { self', system, ... }:
        let
          packageName = "git-z";

          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rust-toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          naersk = pkgs.callPackage inputs.naersk {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };

          mkPackage = { extraCargoBuildOptions ? [ ] }: naersk.buildPackage {
            src = ./.;
            cargoBuildOptions = opts: opts ++ extraCargoBuildOptions;
            RUSTFLAGS = "-Amissing_docs";

            nativeBuildInputs = with pkgs; [ makeWrapper ];

            postInstall = with pkgs; ''
              wrapProgram $out/bin/${packageName} \
                --prefix PATH : ${lib.makeBinPath [ git ]}
            '';
          };

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
          ];

          ideToolchain = with pkgs; [
            nil
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
              name = "TYPOS_LSP_PATH";
              value = "${pkgs.typos-lsp}/bin/typos-lsp";
            }
          ];
        in
        {
          packages = {
            default = self'.packages.${packageName};

            ${packageName} = mkPackage { };

            "${packageName}-unstable" = mkPackage {
              extraCargoBuildOptions = [ "--features unstable-pre-commit" ];
            };
          };

          devshells = {
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

              motd = ''

                {202}🔨 Welcome to the git-z CI environment!{reset}
              '';

              packages =
                buildToolchain
                ++ checkToolchain;

              env =
                testEnv;
            };
          };
        };
    };
}
