#!/bin/bash

set -e

../stage1 ../std . -o incremental
rustc incremental.rs -o incremental
./incremental ../std