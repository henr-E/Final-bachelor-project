{
  description = "Energy Simulator environment flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11-small";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.follows = "rust-overlay/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        inherit (nixpkgs) lib;
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Get the rust toolchain from the `rust-toolchain.toml` file.
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        # Use crane to build rust packages.
        # Also sets the project specific rust toolchain in crane.
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        workspace = {
          pname = "energy-simulator";
          version = "0.0.0";
        };

        # Clean up unnecessary files from sources in order to minimize cache misses in the CI.
        sources = {
          frontend = lib.cleanSourceWith {
            src = craneLib.path ./frontend;
            filter = path: type:
              (type == "directory")
              || (lib.hasSuffix "yarn.lock" path)
              || (builtins.match ".*\\.jsx?$" path != null)
              || (builtins.match ".*\\.mjs$" path != null)
              || (builtins.match ".*\\.json$" path != null)
              || (builtins.match ".*\\.tsx?$" path != null)
              || (builtins.match ".*\\.s?css$" path != null)
              || (builtins.match ".*\\.sh$" path != null)
              || (builtins.match ".*/public/.*" path != null);
          };
          rust = lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              (builtins.match ".*/frontend/.*" path == null)
              && ((craneLib.filterCargoSources path type)
                || (builtins.match ".*\\.proto$" path != null)
                || (builtins.match ".*\\.sql$" path != null)
                || (builtins.match ".*\\.sqlx/.*" path != null)
                # Add all markdown files, except the ones from the `docs/` directory.
                || (lib.hasSuffix ".md" path && (builtins.match ".*/docs/.*" path == null)));
          };
          proto = craneLib.path ./proto;
        };

        # Define arguments that should be the same across all derivations.
        commonArgs = {
          rustBins = {
            src = sources.rust;
            inherit (workspace) pname version;

            # Packages needed at build time.
            nativeBuildInputs = with pkgs;
              [
                pkg-config
                protobuf
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];

            # Packages that can be linked against by compilers.
            buildInputs = with pkgs; [
              openssl
            ];

            # Compile time environment variables.
            ENV_IMPURE = true;
            SQLX_OFFLINE = true;
            DO_MIGRATIONS = false;
          };
          frontend = {};
        };

        # Prefetch/build dependencies.
        dependencies = {
          rustBins = craneLib.buildDepsOnly commonArgs.rustBins;
        };

        # A list of all builds outputs.
        builds = {
          rustBins = craneLib.buildPackage (commonArgs.rustBins
            // {
              cargoArtifacts = dependencies.rustBins;

              doCheck = false;
            });
          frontend =
            (pkgs.mkYarnPackage
              {
                pname = "frontend";
                src = sources.frontend;

                nativeBuildInputs = with pkgs; [
                  pkg-config
                  protobuf
                  bash
                  yarn
                ];
                buildInputs = with pkgs; [
                  vips
                  nodejs-slim_20
                ];

                configurePhase = ''
                  # We can't link as this would for some reason also link the full node modules in
                  # the derivation's output, wasting a significant amount of space in the final
                  # container.
                  cp -r $node_modules node_modules

                  echo "Generating protobuf files..."
                  ./build-proto.sh ${sources.proto}
                '';
                buildPhase = ''
                  export HOME=$(mktemp -d)
                  yarn --offline run build
                '';
                distPhase = "true";
                installPhase = ''
                  mkdir -p $out
                  cp -R .next/standalone/. $out
                  mkdir -p $out/.next/static
                  cp -R .next/static/. $out/.next/static
                  cp -R public $out/public

                  # Create a binary to be able to easily launch the frontend.
                  mkdir -p $out/bin
                  echo "#! ${pkgs.bash}/bin/bash" >> $out/bin/frontend
                  echo "${pkgs.nodejs-slim_20}/bin/node $out/server.js" >> $out/bin/frontend
                  chmod +x $out/bin/frontend
                '';
              })
            .overrideAttrs (arg: {
              # Ensure the entire `node_modules` does not get added.
              disallowedRequisites = [arg.passthru.deps];
            });
        };

        # Generate a container image for each of the services.
        containers = {
          frontend = pkgs.dockerTools.buildImage {
            name = "frontend";
            tag = "latest";

            copyToRoot = builds.frontend;
            config.Cmd = ["/bin/frontend"];
          };
          rustBins = pkgs.dockerTools.buildImage {
            name = "rust-bins";
            tag = "latest";

            copyToRoot = builds.rustBins;
          };
        };

        beLib = self.lib."${system}";
      in {
        formatter = pkgs.alejandra;

        packages =
          # Append `-container` to all the names of the containers.
          (lib.attrsets.concatMapAttrs (name: val: {"${name}-container" = val;}) containers)
          // builds;

        apps = let
          # Helper function that turns a set with paths of derivations into a set of apps.
          mapToApp = builtins.mapAttrs (_: app: {
            program = app;
            type = "app";
          });
        in
          mapToApp {
            # Script that pushes the images from this flake into the docker repository provided as
            # the first argument to the script.
            #
            # If no first argument is provided, the locally running docker daemon is used.
            release =
              (pkgs.writeShellScript "release" (''
                  echo "Publishing images..."

                  REGISTRY=''${2:-"docker-daemon:"}
                  TAG=''${1}

                  if [ -z "$TAG" ]; then
                    echo "ERROR: Please provide an image tag as first argument."
                    exit 1
                  fi

                ''
                + (lib.concatStrings (
                  builtins.attrValues (
                    builtins.mapAttrs (
                      # Map each container into a command to publish said container.
                      name: value: ''
                        echo "Publishing ${lib.strings.toLower name}..."

                        ${pkgs.skopeo}/bin/skopeo copy \
                          --dest-creds "''${3:-null}" \
                          docker-archive://${containers.${name}} \
                          "$(echo $REGISTRY)${lib.strings.toLower name}:$TAG"
                      ''
                    )
                    containers
                  )
                ))))
              .outPath;
          };

        checks =
          {
            # Run `cargo clippy`.
            cargo-clippy = craneLib.cargoClippy (commonArgs.rustBins
              // {
                cargoArtifacts = dependencies.rustBins;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });

            # Check formatting of rust projects.
            cargo-fmt = craneLib.cargoFmt {
              inherit (workspace) pname version;
              src = sources.rust;
            };

            # Check correctness of docs in the rust project.
            cargo-docs = craneLib.cargoDoc (commonArgs.rustBins
              // {
                cargoArtifacts = dependencies.rustBins;

                RUSTDOCFLAGS = "-Dwarnings";
              });

            # Run `cargo test`.
            cargo-test = craneLib.cargoTest (commonArgs.rustBins
              // {
                cargoArtifacts = dependencies.rustBins;
              });

            # Ensure people connect to the database the right way.
            database-url = beLib.mkCheck {
              name = "database-url";
              shellScript = ''
                cd ${sources.rust}

                echo "Looking for incorrect configuring of database urls..."
                ${pkgs.ripgrep}/bin/rg -n "var\(\"DATABASE_URL\"\)" || status=$?
                if [ $status ]; then
                  echo "Check passed!"
                else
                  echo "Oh no! It seems you are incorrectly configuring the database url in one of your crates."
                  echo "Use \`database_config::database_url(\"<DB_NAME>\")\` to connect to the database."
                  exit 1
                fi
              '';
            };
          }
          // builds;

        devShells = {
          default = let
            devToolchain = toolchain.override {
              extensions = ["rust-analyzer" "rust-src"];
            };
          in
            pkgs.mkShell {
              # Re-use dependencies from the build shell.
              inputsFrom = builtins.attrValues builds;
              packages = with pkgs;
                [
                  # Allow build tools to find installed libraries.
                  pkg-config
                  # Add rust toolchain and make sure it has the correct version of rust-analyzer.
                  devToolchain
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
                  # Script to automatically inspect the container image specified.
                  #
                  # Taken from: https://fasterthanli.me/series/building-a-rust-service-with-nix/part-11
                  (pkgs.writeShellScriptBin "inspect-container" ''
                    ${gzip}/bin/gunzip --stdout $1 > /tmp/image.tar && ${dive}/bin/dive docker-archive:///tmp/image.tar
                  '')
                  # visualize `.dot`
                  graphviz
                  # javascript package manager
                  yarn
                  # javascript linter
                  nodePackages.eslint
                ]
                # MacOS specific packages
                ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                  darwin.apple_sdk.frameworks.SystemConfiguration
                ];

              RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";
              shellHook = ''
                # Install pre-commit hooks to the local git repo.
                ${pkgs.pre-commit}/bin/pre-commit install
              '';
            };
        };

        lib = {
          mkCheck = {
            name,
            shellScript,
          }:
            pkgs.runCommand name {} (''
                # Ensure there is ouput for the derivation (nix will complain otherwise).
                mkdir -p $out
              ''
              + shellScript);
        };
      }
    );
}
