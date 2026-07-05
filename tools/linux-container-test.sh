#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${SIKO_LINUX_IMAGE:-siko-linux-test}"
PLATFORM="${SIKO_LINUX_PLATFORM:-linux/amd64}"

if [[ $# -eq 0 ]]; then
    RUN_CMD="make test"
elif [[ "$1" == "--runner-filter" ]]; then
    if [[ $# -ne 2 ]]; then
        echo "usage: $0 --runner-filter <substring>" >&2
        exit 1
    fi
    RUN_CMD="make runner.bin && ./runner.bin $2"
else
    RUN_CMD="$*"
fi

docker build \
    --platform "$PLATFORM" \
    -f "$ROOT/Dockerfile.linux" \
    -t "$IMAGE" \
    "$ROOT"

docker run \
    -i \
    --rm \
    --platform "$PLATFORM" \
    -e SIKO_ROOT=/work \
    -e SIKO_TARGET_OS=linux \
    -v "$ROOT":/src:ro \
    "$IMAGE" \
    bash -s -- "$RUN_CMD" <<'EOF'
set -euo pipefail

run_cmd="$1"

rm -rf /work
mkdir -p /work
tar -C /src \
    --exclude=.git \
    --exclude=target \
    --exclude='*.bin' \
    --exclude=.DS_Store \
    -cf - . | tar -C /work -xf -

cd /work
export SIKO_ROOT=/work
export SIKO_TARGET_OS=linux

echo "+ ${run_cmd}"
bash -lc "$run_cmd"
EOF
