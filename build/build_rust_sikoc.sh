#!/bin/bash

set -e -u

cd `dirname $0`

cd ../rust/

cargo build --release

cp ./target/release/siko ../rust_sikoc
