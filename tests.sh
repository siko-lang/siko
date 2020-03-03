#!/bin/bash

set -e

./build.sh

mkdir -p comp
mkdir -p rust_comp

cd siko_tester
cargo run -- ../siko ../std ../comp ../rust_comp ../tests/success/ ../tests/fail/ ../rt/
