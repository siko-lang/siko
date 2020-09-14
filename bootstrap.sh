#!/bin/bash

mkdir -p bootstrap
echo "Generating stage0"
date
./siko std sikoc -c bootstrap/source.rs
cp rt/main.rs bootstrap/main.rs
echo "Compiling stage0"
date
rustc --edition=2018 bootstrap/main.rs -o bootstrap/stage0 -O
echo "Generating stage1"
date
find ./sikoc -name *.sk | xargs ./bootstrap/stage0 std2/* -o bootstrap/stage1
echo "Compiling stage1"
date
rustc --edition=2018 bootstrap/stage1_rc.rs -O -o bootstrap/stage1
echo "Generating stage2"
date
find ./sikoc -name *.sk | xargs bootstrap/stage1 std2/* -o bootstrap/stage2
echo "Compiling stage2"
date
rustc --edition=2018 bootstrap/stage2_rc.rs -O -o bootstrap/stage2
