#!/bin/bash

set -e

./build.sh

mkdir -p comp
mkdir -p rust_comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../rust_comp ../tests/success/ ../tests/fail/ ../rt/

cd ..

mkdir -p sikoc_test_runs

./sikoc.py sikoc_test_runs/cmp std2 sikoc_tests/success/operators/cmp/main.sk
./sikoc.py sikoc_test_runs/logic std2 sikoc_tests/success/operators/logic/main.sk
