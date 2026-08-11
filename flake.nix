{
  description = "monokaku development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, git-hooks, ... }:
    let
      systems = [ "aarch64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.stable."1.97.1".default;
        in
        {
          default = pkgs.mkShell {
            packages = [
              rustToolchain # rustc / cargo / rustfmt / clippy
              pkgs.cargo-audit
              pkgs.cargo-deny
              pkgs.bacon
            ];
            shellHook = self.checks.${system}.pre-commit-check.shellHook;
          };
        }
      );
      checks = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.stable."1.97.1".default;
        in
        {
          pre-commit-check = git-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt = {
                enable = true;
                package = rustToolchain;
                settings = {
                  check = true;
                };
              };
              clippy = {
                enable = true;
                packageOverrides = {
                  cargo = rustToolchain;
                  clippy = rustToolchain;
                };
                settings = {
                  denyWarnings = true;
                  allFeatures = true;
                };
              };
              cargo-deny = {
                enable = true;
                entry = "${(pkgsFor system).cargo-deny}/bin/cargo-deny check";
                pass_filenames = false;
              };
              cargo-audit = {
                enable = true;
                entry = "${(pkgsFor system).cargo-audit}/bin/cargo-audit audit";
                pass_filenames = false;
              };
            };
          };
        }
      );
    };
}
