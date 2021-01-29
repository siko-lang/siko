#!/bin/bash

set -e

./build.sh

mkdir -p comp
mkdir -p rust_comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../rust_comp ../tests/success/ ../tests/fail/ ../rt/

cd ..

mkdir -p sikoc_test_runs
if [ ! -d compiled_sikoc ];
then
    ./compile_sikoc.sh
fi
./testrunner.py
rm -rf compiled_sikoc
