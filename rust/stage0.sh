#!/bin/bash
set -e

./build.sh
mkdir -p bootstrap/src
echo "Generating stage0"
date
./siko -s ../std ../sikoc/src -c bootstrap/src/source.rs
cp rt/main.rs bootstrap/src/main.rs
cp rt/Cargo.toml bootstrap/Cargo.toml
echo "Compiling stage0"
date
cd bootstrap/
cargo build --release
ln -snf ./target/release/rust_sikoc stage0