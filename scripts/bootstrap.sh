#!/bin/bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$script_dir"

# refresh bootstrap repo after make refresh
cd ..

rm -f ../bootstrap/source*
cp bootstrap/source* ../bootstrap/
cd ../bootstrap
git add -A -- 'source*'
if ! git diff --cached --quiet; then
    git commit -m update
fi
git push
cd -
cd bootstrap/
git fetch origin master
git reset --hard FETCH_HEAD
