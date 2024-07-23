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
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rust-toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          naersk = pkgs.callPackage inputs.naersk {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };

          packageName = "git-z";
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

          devshells.default = {
            name = "git-z";

            motd = ''

              {202}🔨 Welcome to the git-z devshell!{reset}
            '';

            packages = with pkgs; with self'.packages; [
              # Build toolchain
              rust-toolchain
              clang

              # IDE toolchain
              nil
              rust-analyzer

              # Linters and formatters
              committed
              eclint
              nixpkgs-fmt
              taplo
              typos

              # Tools
              cargo-bloat
              cargo-outdated
              cargo-watch
              git
              git-z
              gitAndTools.gitflow
            ];

            env = [
              {
                name = "TYPOS_LSP_PATH";
                value = "${pkgs.typos-lsp}/bin/typos-lsp";
              }
            ];
          };
        };
    };
}
