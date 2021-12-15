#!/bin/bash

set -e -u

cargo build --release

ln -snf ./target/release/siko siko