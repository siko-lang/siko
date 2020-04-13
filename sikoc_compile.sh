#!/bin/bash

set -e

mkdir -p sikoc_compile_test
./siko -c sikoc_compile_test/source.rs sikoc
cp rt/* sikoc_compile_test/
cd sikoc_compile_test
./build.sh
