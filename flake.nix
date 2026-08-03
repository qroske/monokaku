{
  description = "monokaku development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
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
            ];
          };
        }
      );
    };
}
