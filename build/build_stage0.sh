#!/bin/bash

set -e

cd `dirname $0`

mkdir -p stage0/src
echo "Generating stage0"
date
../rust_sikoc -s ../std ../sikoc/src -c stage0/src/source.rs
cp rt/main.rs stage0/src/main.rs
cp rt/Cargo.toml stage0/Cargo.toml
echo "Compiling stage0"
date
cd stage0/
cargo build --release
cp ./target/release/rust_sikoc ../../stage0
