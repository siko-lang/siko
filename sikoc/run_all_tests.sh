#!/bin/bash

set -e

./build.sh

mkdir -p sikoc_test_runs
if [ ! -e compiled_sikoc ];
then
    ./compile_sikoc.sh
fi
./tests.sh