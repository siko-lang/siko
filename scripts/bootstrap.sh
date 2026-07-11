#!/bin/bash

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $script_dir

# refresh bootstrap repo after make refresh
cd ..

rm ../bootstrap/source*
cp bootstrap/source* ../bootstrap/
cd ../bootstrap
git add source*
git commit -m update
git push
cd -
rm bootstrap/source*
cd bootstrap/
git pull origin master