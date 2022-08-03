#!/bin/bash
set -e

echo "Generating stage2"
date
STD2=`ls ../sikoc/std2/*.sk ../sikoc/std2/Json/*.sk`
find ../sikoc/src -name "*.sk" | xargs bootstrap/stage1 $STD2 -v -o bootstrap/stage2
echo "Compiling stage2"
date
rustc --edition=2018 bootstrap/stage2_normal.rs -O -o bootstrap/stage2
