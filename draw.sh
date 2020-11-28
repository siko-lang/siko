#!/bin/bash

for dot in `ls -1 dots/*.dot`; do
    dot -Tpng $dot -o $dot.png
done