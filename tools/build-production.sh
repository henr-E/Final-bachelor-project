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
echo "Running build script..."
mkdir -p .out
docker run -it --cpus $(nproc) -v $NIX_VOLUME:/nix --mount "type=bind,source=$(pwd)/.out,target=/out" $NIX_IMAGE ./tools/build-production/build-script.sh

echo
echo "Loading generated images..."
for f in .out/*; do
    echo "Loading: $f"
    docker load -i $f
done
