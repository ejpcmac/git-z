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
      systems = [ "x86_64-linux" ];

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

          buildToolchain = with pkgs; [
            rust-toolchain
            clang
          ];

          ideToolchain = with pkgs; [
            nil
            rust-analyzer
          ];

          lintersAndFormatters = with pkgs; [
            committed
            eclint
            nixpkgs-fmt
            taplo
            typos
          ];

          tools = with pkgs; with self'.packages; [
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

            ${packageName} = naersk.buildPackage {
              src = ./.;
              RUSTFLAGS = "-Amissing_docs";

              nativeBuildInputs = with pkgs; [ makeWrapper ];

              postInstall = with pkgs; ''
                wrapProgram $out/bin/${packageName} \
                  --prefix PATH : ${lib.makeBinPath [ git ]}
              '';
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
                ++ ideToolchain
                ++ lintersAndFormatters
                ++ tools;

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
                ++ lintersAndFormatters;

              env =
                testEnv;
            };
          };
        };
    };
}
