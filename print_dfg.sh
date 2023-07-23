#!/bin/bash

cd dot

dots=$(find . -name '*.dot')

for d in $dots;
do
    fn=$(basename -- "$d")
    name="${fn%.*}"
    dot -Tpng $name.dot > $name.png
done

