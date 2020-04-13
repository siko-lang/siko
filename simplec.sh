#!/bin/bash

set -e

./build.sh

./siko -c simplec/source.rs simple.sk
cp rt/* simplec/
cd simplec
./build.sh
./alma