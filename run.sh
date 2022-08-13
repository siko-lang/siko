#!/bin/bash

set -e -u

CMD=$1
shift
./$CMD $@
