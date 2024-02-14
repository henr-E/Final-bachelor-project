{
  description = "Energy Simulator environment flake";

  inputs = {
    nixpkgs.url = "nixpkgs/release-23.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.follows = "rust-overlay/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... } @ inputs:
  flake-utils.lib.eachDefaultSystem ( system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          # rust packages
          rust-bin.stable."1.75.0".default
          # real time running of tests/compiling/checking/linting during
          # development
          bacon
        ];
      };
    }
  );
}
