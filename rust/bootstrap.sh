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
cd ..
echo "Generating stage1"
date
STD2=`ls ../sikoc/std2/*.sk ../sikoc/std2/Json/*.sk`
find ../sikoc/src -name "*.sk" | xargs ./bootstrap/target/release/rust_sikoc $STD2 -v -o bootstrap/stage1
echo "Compiling stage1"
date
rustc --edition=2018 bootstrap/stage1_normal.rs -O -o bootstrap/stage1
echo "Generating stage2"
date
find ../sikoc/src -name "*.sk" | xargs bootstrap/stage1 $STD2 -v -o bootstrap/stage2
echo "Compiling stage2"
date
rustc --edition=2018 bootstrap/stage2_normal.rs -O -o bootstrap/stage2
