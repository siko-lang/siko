#!/bin/bash

set -e -u

mkdir -p bootstrap

find std sikoc -name '*.sk' | xargs ./merger.py > bootstrap/sikoc.sk

cd bootstrap
date
../siko -s ../std ../sikoc
date
