#!/bin/bash

scriptdir=`dirname $0`

cd $scriptdir

clang -c siko_runtime.c -o siko_runtime.o