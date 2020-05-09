#!/bin/bash

./siko sikoc

dot *.dot -Tpng -O > /dev/null 2>&1
