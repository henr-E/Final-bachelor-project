# Build the docker image in multiple stages. Stages are as follows:
# -   build-env
# -   rust-build
# -   rust-bins (includes all rust binaries)
# -   frontend

FROM nixos/nix:2.20.4 as build-env

WORKDIR /build-env

COPY ./flake.nix flake.nix
COPY ./flake.lock flake.lock
COPY ./rust-toolchain.toml rust-toolchain.toml

# Prefetch nixpkgs dependencies. This results in a larger image, but faster ci.
RUN nix build .#build \
    --extra-experimental-features nix-command \
    --extra-experimental-features flakes

# Optimise nix store
RUN nix store gc \
    --extra-experimental-features nix-command \
    --extra-experimental-features flakes

ENTRYPOINT [ \
    "nix", \
    "--extra-experimental-features", "nix-command flakes", \
    "develop", \
    ".#build", "--command" \
]
