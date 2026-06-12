#!/bin/bash
# Usage: sikoc build ... --pass c | ./link.sh -o output [-O] [--san] [--tsan]
# Reads C source from stdin, compiles with clang + bdw-gc.
# --san:  AddressSanitizer + UndefinedBehaviorSanitizer (macOS supported)
# --tsan: ThreadSanitizer (mutually exclusive with --san)

output=""
optimize=0
san=0
tsan=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        -o) output="$2"; shift 2 ;;
        -O) optimize=1; shift ;;
        --san) san=1; shift ;;
        --tsan) tsan=1; shift ;;
        *) echo "unknown argument: $1" >&2; exit 1 ;;
    esac
done

if [[ -z "$output" ]]; then
    echo "usage: ... | link.sh -o <output> [-O] [--san] [--tsan]" >&2
    exit 1
fi

if [[ $san -eq 1 && $tsan -eq 1 ]]; then
    echo "error: --san and --tsan are mutually exclusive" >&2
    exit 1
fi

gc_flags=$(pkg-config --cflags --libs bdw-gc 2>/dev/null || echo "-lgc")

opt_flag=""
if [[ $optimize -eq 1 ]]; then
    opt_flag="-O3"
fi

san_flags=""
if [[ $san -eq 1 ]]; then
    san_flags="-fsanitize=address,undefined -fno-sanitize=leak -fno-omit-frame-pointer -g"
elif [[ $tsan -eq 1 ]]; then
    san_flags="-fsanitize=thread -fno-omit-frame-pointer -g"
fi

exec clang -Wno-unused-value -Wno-pointer-sign -Wno-incompatible-pointer-types -x c - -o "$output" $opt_flag $san_flags $gc_flags
