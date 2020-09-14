#!/bin/bash

mkdir -p compiled_sikoc
./siko std sikoc -c compiled_sikoc/source.rs
cp rt/main.rs compiled_sikoc
rustc --edition=2018 compiled_sikoc/main.rs -o compiled_sikoc/sikoc -O