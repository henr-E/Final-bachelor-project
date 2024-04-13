#! /usr/bin/env bash

# Exit on the first error.
set -e

NUM_CPUS=${NUM_CPUS:-$((nproc))}
NIX_IMAGE=${NIX_IMAGE:-"nix"}
NIX_VOLUME=${NIX_VOLUME:-"nix"}

echo "Building nix container..."
docker build -t $NIX_IMAGE -f ./tools/build-production/Dockerfile .

# This extra echo is intentional to add some spacing.
echo
echo "Running pipeline..."
docker run -it --cpus $NUM_CPUS -v $NIX_VOLUME:/nix $NIX_IMAGE nix flake check -L
