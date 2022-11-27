#!/bin/bash

set -e
set -o pipefail

cd `dirname $0`

mkdir -p fmt

SIKOC=../stage1

 ${SIKOC} ../std ../fmt/src -o fmt/sikofmt -v
rustc fmt/sikofmt.rs -o fmt/sikofmt -O
cp fmt/sikofmt ../sikofmt
