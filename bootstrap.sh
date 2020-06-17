#!/bin/bash

set -e -u

mkdir -p bootstrap

cd bootstrap
date
../siko -s ../std ../sikoc -c source.rs
cp ../rt/main.rs .
rustc --edition=2018 main.rs -o sikoc_rust --crate-name sikoc_rust -O
date
