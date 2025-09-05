#!/usr/bin/python3

import os
import subprocess
import sys
import time

# ANSI color codes
GREEN = '\033[92m'
YELLOW = '\033[93m'
RED = '\033[91m'
RESET = '\033[0m'

def red(text):
    return f"{RED}{text}{RESET}"

def green(text):
    return f"{GREEN}{text}{RESET}"

def yellow(text):
    return f"{YELLOW}{text}{RESET}"

success = 0
failure = 0
skipped = 0
in_workflow = False

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

def compileSiko(currentDir, files, extras, isUnitTest):
    bin_output_path = os.path.join(currentDir, "main.bin")
    sanitize = []
    global in_workflow
    if in_workflow:
        sanitize = []
    else:
        sanitize = ["--sanitize"]
    if isUnitTest:
        args = ["./siko", "test"]
    else:
        args = ["./siko", "build"]
    args = args + sanitize + ["-o", bin_output_path] + extras + files
    start_time = time.time()
    if isUnitTest:
        r = subprocess.run(args, stdout=subprocess.DEVNULL)
    else:
        r = subprocess.run(args)
    end_time = time.time()
    compilation_time = end_time - start_time
    if r.returncode != 0:
        return None, compilation_time
    return bin_output_path, compilation_time

def compare_output(output_txt_path, current_output):
    if os.path.exists(output_txt_path):
        with open(output_txt_path, "rb") as f:
            existing_output = f.read()
        if existing_output != current_output:
            print(" - failed")
            print("Expected:")
            for line in existing_output.splitlines():
                print("   ", line.decode())
            print("Got:")
            for line in current_output.splitlines():
                print("   ", line.decode())
            return False
        return existing_output == current_output
    else:
        with open(output_txt_path, "wb") as f:
            f.write(current_output)
        return True

def test_success(root, entry, extras, explicit):
    print("- %s" % entry, end='')
    start_time = time.time()
    currentDir = os.path.join(root, entry)
    skipPath = os.path.join(currentDir, "SKIP")
    if os.path.exists(skipPath) and not explicit:
        end_time = time.time()
        return ("skip", end_time - start_time, 0, 0)
    unitTestPath = os.path.join(currentDir, "TEST")
    isUnitTest = os.path.exists(unitTestPath)
    inputPath = os.path.join(currentDir, "main.sk")
    binary, compilation_time = compileSiko(currentDir, [inputPath], extras, isUnitTest)
    if binary is None:
        end_time = time.time()
        return (False, end_time - start_time, compilation_time, 0)

    execution_start = time.time()
    r = subprocess.run([binary], capture_output=True)
    execution_end = time.time()
    execution_time = execution_end - execution_start

    if r.returncode != 0:
        sys.stdout.write(r.stdout.decode())
        sys.stderr.write(r.stderr.decode())
        end_time = time.time()
        return (False, end_time - start_time, compilation_time, execution_time)
    if in_workflow:
        valgrind_start = time.time()
        if not runUnderValgrind(binary, []):
            valgrind_end = time.time()
            execution_time += valgrind_end - valgrind_start
            end_time = time.time()
            return (False, end_time - start_time, compilation_time, execution_time)
        valgrind_end = time.time()
        execution_time += valgrind_end - valgrind_start
    output_txt_path = os.path.join(root, entry, "output.txt")
    result = compare_output(output_txt_path, r.stdout + r.stderr)
    end_time = time.time()
    return (result, end_time - start_time, compilation_time, execution_time)

def test_fail(root, entry, extras):
    print("- %s" % entry, end = '')
    global success, failure, skipped
    start_time = time.time()
    skip_path = os.path.join(root, entry, "SKIP")
    if os.path.exists(skip_path):
        end_time = time.time()
        return ("skip", end_time - start_time)
    input_path = os.path.join(root, entry, "main.sk")
    args = ["./siko", "build", input_path] + extras
    r = subprocess.run(args, capture_output=True)
    if r.returncode == 0:
        end_time = time.time()
        return (False, end_time - start_time)
    output_txt_path = os.path.join(root, entry, "output.txt")
    result = compare_output(output_txt_path, r.stdout + r.stderr)
    end_time = time.time()
    return (result, end_time - start_time)

filters = []
for arg in sys.argv[1:]:
    if arg == "--workflow":
        in_workflow = True
        continue
    filters.append(arg)

successes_path = os.path.join(".", "test", "success")

errors_path = os.path.join(".", "test", "errors")

failures = []

def processResult(r, name, is_success_test=False):
    global success, failure, skipped
    if is_success_test:
        result, total_runtime, compilation_time, execution_time = r
        total_runtime_ms = total_runtime * 1000
        compilation_time_ms = compilation_time * 1000
        execution_time_ms = execution_time * 1000
        if result == "skip":
            print(" - %s (%.1fms)" % (yellow("SKIPPED"), total_runtime_ms), flush=True)
            skipped += 1
            return
        if result:
            print(" - %s (compile: %.1fms, exec: %.1fms, total: %.1fms)" % (green("success"), compilation_time_ms, execution_time_ms, total_runtime_ms), flush=True)
            success += 1
        else:
            print(" - %s (compile: %.1fms, exec: %.1fms, total: %.1fms)" % (red("failed"), compilation_time_ms, execution_time_ms, total_runtime_ms), flush=True)
            failure += 1
            failures.append(name)
    else:
        result, runtime = r
        runtime_ms = runtime * 1000
        if result == "skip":
            print(" - SKIPPED (%.1fms)" % runtime_ms, flush=True)
            skipped += 1
            return
        if result:
            print(" - %s (%.1fms)" % (green("success"), runtime_ms), flush=True)
            success += 1
        else:
            print(" - %s (%.1fms)" % (red("failed"), runtime_ms), flush=True)
            failure += 1
            failures.append(name)

def collect_tests(base_path):
    tests = []
    for root, dirs, files in os.walk(base_path):
        if any(file.endswith(".sk") for file in files):
            tests.append(root)
    tests.sort()
    return tests

print("Success tests:")
success_tests = collect_tests(successes_path)
for test_path in success_tests:
    entry = os.path.relpath(test_path, successes_path)
    if filters and entry not in filters:
        continue
    processResult(test_success(successes_path, entry, ["std"], entry in filters), entry, True)

print("Error tests:")
failed_tests = collect_tests(errors_path)
for test_path in failed_tests:
    entry = os.path.relpath(test_path, errors_path)
    if len(filters) > 0 and entry not in filters:
        continue
    processResult(test_fail(errors_path, entry, ["std"]), entry, False)

percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success: %s failure: %s skip: %s - %.2f%%" % (success, failure, skipped, percent))

if failure > 0:
    print("Failures:")
    for f in failures:
        print(f)
    sys.exit(1)
