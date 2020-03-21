#!/bin/bash

set -e -u

mkdir -p bootstrap

find std sikoc -name '*.sk' | xargs -I FOO cat FOO > bootstrap/sikoc.sk

cd bootstrap
date
../siko -s ../std ../sikoc
date
