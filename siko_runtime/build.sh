#!/bin/bash

scriptdir=`dirname $0`

cd $scriptdir

clang -c siko_runtime.c -o siko_runtime.o -I .
#clang -c -emit-llvm siko_runtime.c -o siko_runtime.bc
#clang -c -S -emit-llvm siko_runtime.c -o siko_runtime.ll