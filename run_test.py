#!/usr/bin/python3

import argparse
import concurrent.futures
import os
import subprocess
import sys
import threading
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
output_lock = threading.Lock()

class TestResult:
    def __init__(self, name, result, time_data, is_success_test=False, output=""):
        self.name = name
        self.result = result
        self.time_data = time_data
        self.is_success_test = is_success_test
        self.output = output

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

def compileSiko(currentDir, files):
    bin_output_path = os.path.join(currentDir, "main.bin")
    sanitize = []
    global in_workflow
    if in_workflow:
        sanitize = []
    else:
        sanitize = ["--sanitize"]

    args = ["./siko", "build"]
    args = args + sanitize + ["-o", bin_output_path] + files
    start_time = time.time()
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

def test_success(root, entry, explicit, parallel=False):
    output_buffer = []
    def capture_print(*args, **kwargs):
        if parallel:
            output_buffer.append(' '.join(str(arg) for arg in args))
        else:
            print(*args, **kwargs)

    capture_print("- %s" % entry, end='')
    start_time = time.time()
    currentDir = os.path.join(root, entry)
    skipPath = os.path.join(currentDir, "SKIP")
    if os.path.exists(skipPath) and not explicit:
        end_time = time.time()
        result = TestResult(entry, "skip", (end_time - start_time, 0, 0), True, '\n'.join(output_buffer))
        return result
    inputPath = os.path.join(currentDir, "main.sk")
    binary, compilation_time = compileSiko(currentDir, [inputPath])
    if binary is None:
        end_time = time.time()
        result = TestResult(entry, False, (end_time - start_time, compilation_time, 0), True, '\n'.join(output_buffer))
        return result

    execution_start = time.time()
    r = subprocess.run([binary], capture_output=True)
    execution_end = time.time()
    execution_time = execution_end - execution_start

    if r.returncode != 0:
        if not parallel:
            sys.stdout.write(r.stdout.decode())
            sys.stderr.write(r.stderr.decode())
        end_time = time.time()
        result = TestResult(entry, False, (end_time - start_time, compilation_time, execution_time), True, '\n'.join(output_buffer))
        return result
    if in_workflow:
        valgrind_start = time.time()
        if not runUnderValgrind(binary, []):
            valgrind_end = time.time()
            execution_time += valgrind_end - valgrind_start
            end_time = time.time()
            result = TestResult(entry, False, (end_time - start_time, compilation_time, execution_time), True, '\n'.join(output_buffer))
            return result
        valgrind_end = time.time()
        execution_time += valgrind_end - valgrind_start
    output_txt_path = os.path.join(root, entry, "output.txt")
    result_bool = compare_output(output_txt_path, r.stdout + r.stderr)
    end_time = time.time()
    result = TestResult(entry, result_bool, (end_time - start_time, compilation_time, execution_time), True, '\n'.join(output_buffer))
    return result

def test_fail(root, entry, parallel=False):
    output_buffer = []
    def capture_print(*args, **kwargs):
        if parallel:
            output_buffer.append(' '.join(str(arg) for arg in args))
        else:
            print(*args, **kwargs)

    capture_print("- %s" % entry, end='')
    global success, failure, skipped
    start_time = time.time()
    skip_path = os.path.join(root, entry, "SKIP")
    if os.path.exists(skip_path):
        end_time = time.time()
        result = TestResult(entry, "skip", (end_time - start_time,), False, '\n'.join(output_buffer))
        return result
    input_path = os.path.join(root, entry, "main.sk")
    args = ["./siko", "build", input_path]
    envs = os.environ.copy()
    envs["SIKO_STD_PATH"] = "./std"
    r = subprocess.run(args, capture_output=True, env=envs)
    if r.returncode == 0:
        end_time = time.time()
        result = TestResult(entry, False, (end_time - start_time,), False, '\n'.join(output_buffer))
        return result
    output_txt_path = os.path.join(root, entry, "output.txt")
    result_bool = compare_output(output_txt_path, r.stdout + r.stderr)
    end_time = time.time()
    result = TestResult(entry, result_bool, (end_time - start_time,), False, '\n'.join(output_buffer))
    return result

def parse_args():
    parser = argparse.ArgumentParser(description='Run test suite for Siko compiler')
    parser.add_argument('--no-parallel', action='store_true',
                      help='Run tests sequentially instead of in parallel (default is parallel)')
    parser.add_argument('--workflow', action='store_true',
                      help='Run in workflow mode (no sanitizers, use valgrind)')
    parser.add_argument('tests', nargs='*',
                      help='Specific tests to run (if none specified, run all tests)')
    return parser.parse_args()

args = parse_args()
filters = args.tests
in_workflow = args.workflow
use_parallel = not args.no_parallel  # Default to parallel, unless --no-parallel is specified

successes_path = os.path.join(".", "test", "success")

errors_path = os.path.join(".", "test", "errors")

failures = []

def processResult(test_result):
    global success, failure, skipped
    with output_lock:
        # Print any captured output first
        if test_result.output:
            print(test_result.output, end='')

        if test_result.is_success_test:
            total_runtime, compilation_time, execution_time = test_result.time_data
            total_runtime_ms = total_runtime * 1000
            compilation_time_ms = compilation_time * 1000
            execution_time_ms = execution_time * 1000
            if test_result.result == "skip":
                print(" - %s (%.1fms)" % (yellow("SKIPPED"), total_runtime_ms), flush=True)
                skipped += 1
                return
            if test_result.result:
                print(" - %s (compile: %.1fms, exec: %.1fms, total: %.1fms)" % (green("success"), compilation_time_ms, execution_time_ms, total_runtime_ms), flush=True)
                success += 1
            else:
                print(" - %s (compile: %.1fms, exec: %.1fms, total: %.1fms)" % (red("failed"), compilation_time_ms, execution_time_ms, total_runtime_ms), flush=True)
                failure += 1
                failures.append(test_result.name)
        else:
            runtime, = test_result.time_data
            runtime_ms = runtime * 1000
            if test_result.result == "skip":
                print(" - SKIPPED (%.1fms)" % runtime_ms, flush=True)
                skipped += 1
                return
            if test_result.result:
                print(" - %s (%.1fms)" % (green("success"), runtime_ms), flush=True)
                success += 1
            else:
                print(" - %s (%.1fms)" % (red("failed"), runtime_ms), flush=True)
                failure += 1
                failures.append(test_result.name)

def collect_tests(base_path):
    tests = []
    for root, dirs, files in os.walk(base_path):
        if any(file.endswith(".sk") for file in files):
            tests.append(root)
    tests.sort()
    return tests

def should_run_test(entry, filters):
    """Check if a test should run based on filters. Supports partial matching."""
    if not filters:
        return True

    # Check for exact match first
    if entry in filters:
        return True

    # Check for partial match (if any filter is a prefix of the test path)
    for filter_name in filters:
        if entry.startswith(filter_name + "/") or entry == filter_name:
            return True

    return False

def is_explicit_test(entry, filters):
    """Check if a test was explicitly requested (exact match)."""
    return entry in filters if filters else False

def run_tests_sequential(test_func, base_path, tests):
    """Run tests sequentially"""
    for test_path in tests:
        entry = os.path.relpath(test_path, base_path)
        if not should_run_test(entry, filters):
            continue
        if test_func == test_success:
            result = test_func(base_path, entry, is_explicit_test(entry, filters), parallel=False)
        else:
            result = test_func(base_path, entry, parallel=False)
        processResult(result)

def run_tests_parallel(test_func, base_path, tests):
    """Run tests in parallel"""
    test_args = []
    for test_path in tests:
        entry = os.path.relpath(test_path, base_path)
        if not should_run_test(entry, filters):
            continue
        if test_func == test_success:
            test_args.append((base_path, entry, is_explicit_test(entry, filters), True))
        else:
            test_args.append((base_path, entry, True))

    def run_single_test(args):
        if test_func == test_success:
            return test_func(*args)
        else:
            return test_func(*args)

    # Use ThreadPoolExecutor for parallel execution
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count()) as executor:
        # Submit all tests
        future_to_test = {executor.submit(run_single_test, test_args): test_args for test_args in test_args}

        # Process results as they complete
        for future in concurrent.futures.as_completed(future_to_test):
            try:
                result = future.result()
                processResult(result)
            except Exception as exc:
                print(f'Test generated an exception: {exc}')
                sys.exit(1)

print("Success tests:")
success_tests = collect_tests(successes_path)

if use_parallel:
    run_tests_parallel(test_success, successes_path, success_tests)
else:
    run_tests_sequential(test_success, successes_path, success_tests)

print("Error tests:")
failed_tests = collect_tests(errors_path)

if use_parallel:
    run_tests_parallel(test_fail, errors_path, failed_tests)
else:
    run_tests_sequential(test_fail, errors_path, failed_tests)

percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success: %s failure: %s skip: %s - %.2f%%" % (success, failure, skipped, percent))

if failure > 0:
    print("Failures:")
    for f in failures:
        print(f)
    sys.exit(1)
