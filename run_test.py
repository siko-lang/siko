#!/usr/bin/python3

import os
import subprocess
import sys

success = 0
failure = 0
skipped = 0
in_workflow = False
clang_args = "-fsanitize=undefined,address,alignment,null,bounds,integer,enum,implicit-conversion,float-cast-overflow,float-divide-by-zero"

def runUnderValgrind(program, args):
    valgrind_args = ["valgrind", "--leak-check=full", "--track-origins=yes", "--error-exitcode=1"]
    valgrind_args += [program] + args
    r = subprocess.run(valgrind_args, capture_output=True)
    if r.returncode != 0:
        print("Valgrind error:")
        print(r.stdout.decode())
        print(r.stderr.decode())
        return False
    return True

def compileSikoC(currentDir, files, extras):
    output_path = os.path.join(currentDir, "main")
    c_output_path = os.path.join(currentDir, "main.c")
    object_path = os.path.join(currentDir, "main.o")
    bin_output_path = os.path.join(currentDir, "main.bin")
    args = ["./siko", "-o", output_path] + extras + files
    global clang_args
    if in_workflow:
        clang_args = ""
    r = subprocess.run(args)
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", clang_args, "-g", "-O1", "-c", c_output_path, "-o", object_path, "-I", "siko_runtime"])
    if r.returncode != 0:
        return None
    r = subprocess.run(["clang", clang_args, object_path, "-o", bin_output_path])
    #r = subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        return None
    return bin_output_path

def compare_output(output_txt_path, current_output):
    if os.path.exists(output_txt_path):
        with open(output_txt_path, "rb") as f:
            existing_output = f.read()
        if existing_output != current_output:
            print(" - failed")
            print("Expected:")
            print(existing_output)
            print("Got:")
            print(current_output)
            return False
        return existing_output == current_output
    else:
        with open(output_txt_path, "wb") as f:
            f.write(current_output)
        return True

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
    r = subprocess.run([binary], capture_output=True)
    if r.returncode != 0:
        sys.stdout.write(r.stdout.decode())
        sys.stderr.write(r.stderr.decode())
        return False
    if in_workflow:
        if not runUnderValgrind(binary, []):
            return False
    output_txt_path = os.path.join(root, entry, "output.txt")
    return compare_output(output_txt_path, r.stdout + r.stderr)

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
    return compare_output(output_txt_path, r.stdout + r.stderr)

filters = []
for arg in sys.argv[1:]:
    if arg == "--workflow":
        in_workflow = True
        continue
    filters.append(arg)

successes_path = os.path.join(".", "test", "success")

errors_path = os.path.join(".", "test", "errors")

failures = []

def processResult(r, name):
    global success, failure, skipped
    if r == "skip":
        print(" - SKIPPED", flush=True)
        skipped += 1
        return
    if r:
        print(" - success", flush=True)
        success += 1
    else:
        print(" - failed", flush=True)
        failure += 1
        failures.append(name)

def collect_tests(base_path):
    tests = []
    for root, dirs, files in os.walk(base_path):
        if any(file.endswith(".sk") for file in files):
            tests.append(root)
    return tests

print("Success tests:")
success_tests = collect_tests(successes_path)
for test_path in success_tests:
    entry = os.path.relpath(test_path, successes_path)
    if filters and entry not in filters:
        continue
    processResult(test_success(successes_path, entry, ["std"]), entry)

print("Error tests:")
failed_tests = collect_tests(errors_path)
for test_path in failed_tests:
    entry = os.path.relpath(test_path, errors_path)
    if len(filters) > 0 and entry not in filters:
        continue
    processResult(test_fail(errors_path, entry, ["std"]), entry)

percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success: %s failure: %s skip: %s - %.2f%%" % (success, failure, skipped, percent))

if failure > 0:
    print("Failures:")
    for f in failures:
        print(f)
    sys.exit(1)
