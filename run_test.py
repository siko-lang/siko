#!/usr/bin/python3

import os
import subprocess
import sys

success = 0
failure = 0
skipped = 0

runtimePath = os.path.join("siko_runtime", "siko_runtime.o")

def compileSiko(currentDir, files, extras):
    output_path = os.path.join(currentDir, "main.ll")
    optimized_path = os.path.join(currentDir, "main_optimized.ll")
    bitcode_path = os.path.join(currentDir, "main.bc")
    object_path = os.path.join(currentDir, "main.o")
    llvm_output_path = os.path.join(currentDir, "main.bin")
    args = ["./siko", "-o", output_path] + extras + files
    r = subprocess.run(args)
    if r.returncode != 0:
        return None
    r = subprocess.run(["opt", "-O2", "-S", output_path, "-o", optimized_path])
    if r.returncode != 0:
        return None
    r = subprocess.run(["llvm-as", optimized_path, "-o", bitcode_path])
    if r.returncode != 0:
        return None
    r = subprocess.run(["llc",  bitcode_path, "-filetype=obj", "-o", object_path])
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", object_path, runtimePath, "-o", llvm_output_path])
    #r = subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        return None
    return llvm_output_path

def test(root, entry, extras):
    print("- %s" % entry)
    global success, failure, skipped
    currentDir = os.path.join(root, entry)
    skipPath = os.path.join(currentDir, "SKIP")
    if os.path.exists(skipPath):
        skipped += 1
        return
    inputPath = os.path.join(currentDir, "main.sk")
    binary = compileSiko(currentDir, [inputPath], extras)
    if binary is None:
        failure += 1
        return
    r = subprocess.run([binary])
    if r.returncode != 0:
        failure += 1
        return
    success += 1

def test_fail(root, entry, extras):
    print("- %s" % entry)
    global success, failure, skipped
    skip_path = os.path.join(root, entry, "SKIP")
    if os.path.exists(skip_path):
        skipped += 1
        return
    input_path = os.path.join(root, entry, "main.sk")
    output_path = os.path.join(root, entry, "main.ll")
    args = ["./siko", input_path, "-o", output_path] + extras
    #print(args)
    r = subprocess.run(args, capture_output=True)
    if r.returncode == 0:
        failure += 1
        return
    output_txt_path = os.path.join(root, entry, "output.txt")
    f = open(output_txt_path, "wb")
    f.write(r.stdout)
    f.close()
    success += 1

filters = []
for arg in sys.argv[1:]:
    filters.append(arg)

no_std_path = os.path.join(".", "test", "no_std")

std_path = os.path.join(".", "test", "std")

errors_path = os.path.join(".", "test", "errors")

def buildRuntime():
    subprocess.run("siko_runtime/build.sh")

buildRuntime()

print("No std tests:")
for entry in os.listdir(no_std_path):
    if len(filters) > 0 and entry not in filters:
        continue
    test(no_std_path, entry, [])
print("Std tests:")
for entry in os.listdir(std_path):
    if len(filters) > 0 and entry not in filters:
        continue
    test(std_path, entry, ["std"])
for entry in os.listdir(errors_path):
    if len(filters) > 0 and entry not in filters:
        continue
    test_fail(errors_path, entry, ["std"])
percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success %s/%s/%s - %.2f%%" % (success, success + failure, skipped, percent))
