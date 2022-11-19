#!/bin/bash

set -e

SIKOC=../stage1

 ${SIKOC} ../std src -o incremental -v
rustc incremental.rs -o incremental -O
./incremental ../std src
#./incremental test/mini.sk