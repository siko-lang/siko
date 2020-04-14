#!/bin/bash

#set -e

mkdir -p sikoc_compile_test
./siko -c sikoc_compile_test/source.rs sikoc
cp rt/* sikoc_compile_test/
find std sikoc -name '*.sk' | xargs ./merger.py > sikoc_compile_test/sikoc.sk
cd sikoc_compile_test
./build.sh
date
./alma
date
