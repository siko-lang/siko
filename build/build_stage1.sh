#!/bin/bash

set -e

cd `dirname $0`

mkdir -p stage1
echo "Generating stage1"
date
../stage0 ../sikoc/src ../sikoc/std2 -v -o stage1/stage1 | ts '[%Y-%m-%d %H:%M:%S]' > stage1/log.txt
echo "Compiling stage1"
date
rustc --edition=2018 stage1/stage1.rs -O -o stage1/stage1
cp stage1/stage1 ../stage1
