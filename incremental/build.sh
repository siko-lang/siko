#!/bin/bash

set -e

SIKOC=../stage1

 ${SIKOC} ../std src -o incremental -v
./incremental ../std ./test/mini.sk
#./incremental test/mini.sk