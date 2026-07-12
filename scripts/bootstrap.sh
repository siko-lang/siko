#!/bin/bash

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $script_dir

# refresh bootstrap repo after make refresh
cd ..

rm -f ../bootstrap/source*
cp bootstrap/source* ../bootstrap/
cd ../bootstrap
git add -A -- 'source*'
git commit -m update
git push
cd -
rm -f bootstrap/source*
cd bootstrap/
git pull origin master
