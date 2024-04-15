#! /usr/bin/env bash

mkdir -p /out

echo "Building frontend..."
nix build .#frontend-container -L
rm -f /out/frontend
cp $(readlink result) /out/frontend
echo "Building rust-bins..."
nix build .#rust-bins-container -L
rm -f /out/rust-bins
cp $(readlink result) /out/rust-bins

echo "Done! Made the following images:"
ls /out
