#!/bin/bash

set -e

./build.sh

mkdir -p comp
mkdir -p rust_comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../rust_comp ../tests/success/ ../tests/fail/ ../rt/

cd ..

mkdir -p sikoc_test_runs

./siko.py sikoc_test_runs/cmp std2 sikoc_tests/success/operators/cmp/main.sk
./siko.py sikoc_test_runs/logic std2 sikoc_tests/success/operators/logic/main.sk
./siko.py sikoc_test_runs/math std2 sikoc_tests/success/operators/math/main.sk
./siko.py sikoc_test_runs/pipe std2 sikoc_tests/success/operators/pipe/main.sk
./siko.py sikoc_test_runs/adt std2 sikoc_tests/success/syntax/adt/main.sk
./siko.py sikoc_test_runs/caseof std2 sikoc_tests/success/syntax/caseof/main.sk
./siko.py sikoc_test_runs/deriving std2 sikoc_tests/success/syntax/deriving/main.sk
