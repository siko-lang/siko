#!/usr/bin/env python3
"""Link bootstrap object files into base.bin using clang."""

import argparse
import os
import subprocess
import sys


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Link bootstrap object files into an executable."
    )
    parser.add_argument("-o", "--output", required=True)
    parser.add_argument("objects", nargs="+")
    args = parser.parse_args(argv)

    invalid_objects = [obj for obj in args.objects if not obj.endswith(".o")]
    if invalid_objects:
        parser.error(
            "expected object file inputs ending in .o: " + ", ".join(invalid_objects)
        )

    return args


def gc_lib_args():
    result = subprocess.run(
        ["pkg-config", "--libs", "bdw-gc"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
    )
    gc_flags = result.stdout if result.returncode == 0 else "-lgc"
    return gc_flags.split()


def llvm_lib_args():
    candidates = []
    if os.environ.get("LLVM_CONFIG"):
        candidates.append(os.environ["LLVM_CONFIG"])
    candidates.append("llvm-config")
    if sys.platform == "darwin":
        candidates.append("/opt/homebrew/opt/llvm/bin/llvm-config")

    for candidate in candidates:
        try:
            result = subprocess.run(
                [candidate, "--libdir"],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                check=False,
            )
        except FileNotFoundError:
            continue

        if result.returncode == 0:
            libdir = result.stdout.strip()
            return [f"-L{libdir}", "-lLLVM", f"-Wl,-rpath,{libdir}"]

    return []


def main(argv):
    args = parse_args(argv)
    clang_args = [
        "clang",
        *args.objects,
        "-o",
        args.output,
        *gc_lib_args(),
        *llvm_lib_args(),
    ]
    os.execvp(clang_args[0], clang_args)


if __name__ == "__main__":
    main(sys.argv[1:])
