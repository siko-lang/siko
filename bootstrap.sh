#!/bin/bash
set -e

./build.sh
mkdir -p bootstrap/src
echo "Generating stage0"
date
./siko std sikoc -c bootstrap/src/source.rs
cp rt/main.rs bootstrap/src/main.rs
cp rt/Cargo.toml bootstrap/Cargo.toml
echo "Compiling stage0"
date
cd bootstrap/
cargo build --release
cd ..
echo "Generating stage1"
date
find ./sikoc -name "*.sk" | xargs ./bootstrap/target/release/rust_sikoc std2/* -v -o bootstrap/stage1
echo "Compiling stage1"
date
rustc --edition=2018 bootstrap/stage1_rc.rs -O -o bootstrap/stage1
echo "Generating stage2"
date
find ./sikoc -name "*.sk" | xargs bootstrap/stage1 std2/* -v -o bootstrap/stage2
echo "Compiling stage2"
date
rustc --edition=2018 bootstrap/stage2_rc.rs -O -o bootstrap/stage2
