#!/usr/bin/python3

import os
import subprocess
import sys

success = 0
failure = 0
skipped = 0

runtimePath = os.path.join("siko_runtime", "siko_runtime.o")

def compileSikoLLVM(currentDir, files, extras):
    output_path = os.path.join(currentDir, "main")
    llvm_ir_output_path = os.path.join(currentDir, "main.ll")
    optimized_path = os.path.join(currentDir, "main_optimized.ll")
    bitcode_path = os.path.join(currentDir, "main.bc")
    object_path = os.path.join(currentDir, "main.o")
    llvm_output_path = os.path.join(currentDir, "main.bin")
    args = ["./siko", "-o", output_path] + extras + files
    r = subprocess.run(args)
    if r.returncode != 0:
        return None
    r = subprocess.run(["opt", "-passes=verify", "-S", llvm_ir_output_path, "-o", "/dev/null"])
    if r.returncode != 0:
       return None
    r = subprocess.run(["opt", "-O2", "-S", llvm_ir_output_path, "-o", optimized_path])
    if r.returncode != 0:
       return None
    r = subprocess.run(["llvm-as", optimized_path, "-o", bitcode_path])
    if r.returncode != 0:
        return None
    r = subprocess.run(["llc", "-O0", "-relocation-model=pic", bitcode_path, "-filetype=obj", "-o", object_path])
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", "-O0", object_path, runtimePath, "-o", llvm_output_path])
    #r = subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        return None
    return llvm_output_path

def compileSikoC(currentDir, files, extras):
    output_path = os.path.join(currentDir, "main")
    c_output_path = os.path.join(currentDir, "main.c")
    object_path = os.path.join(currentDir, "main.o")
    bin_output_path = os.path.join(currentDir, "main.bin")
    args = ["./siko", "-o", output_path] + extras + files
    r = subprocess.run(args)
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", "-fsanitize=undefined,address", "-g", "-O1", "-c", c_output_path, "-o", object_path, "-I", "siko_runtime"])
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", "-fsanitize=undefined,address", object_path, runtimePath, "-o", bin_output_path])
    #r = subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        return None
    return bin_output_path

def test_success(root, entry, extras):
    print("- %s" % entry, end='')
    currentDir = os.path.join(root, entry)
    skipPath = os.path.join(currentDir, "SKIP")
    if os.path.exists(skipPath):
        return "skip"
    inputPath = os.path.join(currentDir, "main.sk")
    #binary = compileSikoLLVM(currentDir, [inputPath], extras)
    binary = compileSikoC(currentDir, [inputPath], extras)
    if binary is None:
        return False
    r = subprocess.run([binary])
    if r.returncode != 0:
        return False
    return True

def test_fail(root, entry, extras):
    print("- %s" % entry, end = '')
    global success, failure, skipped
    skip_path = os.path.join(root, entry, "SKIP")
    if os.path.exists(skip_path):
        return "skip"
    input_path = os.path.join(root, entry, "main.sk")
    output_path = os.path.join(root, entry, "main.ll")
    args = ["./siko", input_path, "-o", output_path] + extras
    #print(args)
    r = subprocess.run(args, capture_output=True)
    if r.returncode == 0:
        return False
    output_txt_path = os.path.join(root, entry, "output.txt")
    f = open(output_txt_path, "wb")
    f.write(r.stdout)
    f.close()
    return True

filters = []
for arg in sys.argv[1:]:
    filters.append(arg)

successes_path = os.path.join(".", "test", "success")

errors_path = os.path.join(".", "test", "errors")

def buildRuntime():
    subprocess.run("siko_runtime/build.sh")

def processResult(r):
    global success, failure, skipped
    if r == "skip":
        print(" - SKIPPED")
        skipped += 1
        return
    if r:
        print(" - success")
        success += 1
    else:
        print(" - failed")
        failure += 1

buildRuntime()

print("Success tests:")
for entry in os.listdir(successes_path):
    if len(filters) > 0 and entry not in filters:
        continue
    processResult(test_success(successes_path, entry, ["std"]))
print("Error tests:")
for entry in os.listdir(errors_path):
    if len(filters) > 0 and entry not in filters:
        continue
    processResult(test_fail(errors_path, entry, ["std"]))
percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success %s/%s/%s - %.2f%%" % (success, success + failure, skipped, percent))
