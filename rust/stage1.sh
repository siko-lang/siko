#!/bin/bash
set -e

echo "Generating stage1"
date
STD2=`ls ../sikoc/std2/*.sk ../sikoc/std2/Json/*.sk`
find ../sikoc/src -name "*.sk" | xargs ./bootstrap/stage0 $STD2 -v -o bootstrap/stage1
echo "Compiling stage1"
date
rustc --edition=2018 bootstrap/stage1_normal.rs -O -o bootstrap/stage1
