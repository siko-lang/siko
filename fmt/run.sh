#!/bin/bash

set -e

SIKOC=../stage1

 ${SIKOC} ../std src -o fmt -v
rustc fmt.rs -o fmt -O
./fmt ../std
