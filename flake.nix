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
  };

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devshell.flakeModule ];
      systems = [ "x86_64-linux" ];

      perSystem = { system, ... }:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };
          rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          devshells.default = {
            name = "git-z";

            motd = ''

              {202}🔨 Welcome to the git-z devshell!{reset}
            '';

            packages = with pkgs; [
              # Build toolchain.
              rust-toolchain
              clang
            ];
          };
        };
    };
}
