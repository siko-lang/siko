#!/bin/bash

set -e

mkdir -p compiled_sikoc/src
./siko -s ../std ../sikoc/src -c compiled_sikoc/src/source.rs
cp rt/main.rs compiled_sikoc/src/
cp rt/Cargo.toml compiled_sikoc/
cd compiled_sikoc
cargo build --release
cd ..
ln -sf target/release/rust_sikoc compiled_sikoc/sikoc