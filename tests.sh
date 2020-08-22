#!/bin/bash

set -e

./build.sh

mkdir -p comp
mkdir -p rust_comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../rust_comp ../tests/success/ ../tests/fail/ ../rt/

cd ..

mkdir -p sikoc_test_runs
./compile_sikoc.py
./testrunner.py sikoc_tests
