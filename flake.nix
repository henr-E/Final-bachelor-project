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

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        formatter = pkgs.alejandra;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              # rust packages
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              sqlx-cli
              # rust lsp
              rust-analyzer
              # real time running of tests/compiling/checking/linting during
              # development
              bacon
              # nodejs packages
              nodejs_20
              # run checks and tasks when making a commit
              pre-commit
              # containerization of services for easy development and deployment
              docker
              # used to compile `.proto` files
              protobuf
              # visualize `.dot`
              graphviz
            ]
            # MacOS specific packages
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              darwin.apple_sdk.frameworks.SystemConfiguration
              pkg-config
            ];
          shellHook = ''
            # Install pre-commit hooks to the local git repo.
            ${pkgs.pre-commit}/bin/pre-commit install
          '';
        };
      }
    );
}
