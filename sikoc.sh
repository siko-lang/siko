#!/bin/bash

./siko.py sikocwd std2 sikoc.sk

dot *.dot -Tpng -O > /dev/null 2>&1
