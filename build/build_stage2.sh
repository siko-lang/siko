#!/bin/bash

set -e

cd `dirname $0`

mkdir -p stage2
echo "Generating stage2"
date
#../stage1 ../src ../std -v -o stage2/stage2 | ts '[%Y-%m-%d %H:%M:%S]' > stage2/log.txt
../stage1 ../src ../std -v -o stage2/stage2 | ts '[%Y-%m-%d %H:%M:%S]'
echo "Compiling stage2"
date
rustc --edition=2018 stage2/stage2.rs -O -o stage2/stage2
cp stage2/stage2 ../stage2
