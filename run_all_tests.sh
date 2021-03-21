#!/bin/bash

set -e

./build.sh

mkdir -p comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../tests/success/ ../tests/fail/ ../rt/

cd ..

mkdir -p sikoc_test_runs
if [ ! -e compiled_sikoc ];
then
    ./compile_sikoc.sh
fi
./run_tests.py
rm -rf compiled_sikoc
