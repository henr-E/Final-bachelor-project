{
  description = "Energy Simulator environment flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11-small";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.follows = "rust-overlay/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        buildDeps = with pkgs; [
          protobuf
          pkg-config
          openssl
        ];

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in rec {
        formatter = pkgs.alejandra;

        packages = {
          inherit (self.devShells."${system}") build build-rust build-node;
        };

        devShells = {
          # Shell needed to build rust projects.
          build-rust = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [
                toolchain
                cargo-chef
              ]
              ++ buildDeps;
          };
          # Shell needed to build nodejs projects.
          build-node = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [
                nodejs_20
              ]
              ++ buildDeps;
          };
          # Combines all build shells into one.
          build = pkgs.mkShell {
            inputsFrom = [devShells.build-rust devShells.build-node];
          };
          default = pkgs.mkShell {
            # Re-use dependencies from the build shell.
            inputsFrom = [devShells.build];
            buildInputs = with pkgs;
              [
                # Add rust toolchain and make sure it has the correct version of rust-analyzer.
                (toolchain.override {
                  extensions = ["rust-analyzer" "rust-src"];
                })
                # rust packages
                sqlx-cli
                # js/ts lsp
                nodePackages.typescript-language-server
                # html/css lsp
                vscode-langservers-extracted
                # real time running of tests/compiling/checking/linting during
                # development
                bacon
                # run checks and tasks when making a commit
                pre-commit
                # containerization of services for easy development and deployment
                docker
                # sqlx prepare script
                python310
                # visualize `.dot`
                graphviz
              ]
              # MacOS specific packages
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];
            shellHook = ''
              # Install pre-commit hooks to the local git repo.
              ${pkgs.pre-commit}/bin/pre-commit install
            '';
          };
        };
      }
    );
}
