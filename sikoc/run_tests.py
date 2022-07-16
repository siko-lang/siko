#!/usr/bin/python3

import sys
import os
import errno
import subprocess
import time

def processFile(file):
    content = ""
    f = open(file)
    lines = f.readlines()
    for line in lines:
        l = line.rstrip()
        content += l + "\n"
    last = lines[-1]
    last = last.rstrip()
    if last != "":
        content += "\n"
    return content

def collect_tests(path, tests, parent):
    if os.path.isdir(path):
        files = os.listdir(path)
        found = None
        skip = False
        for f in files:
            full_path = os.path.join(path, f)
            if os.path.isdir(full_path):
                collect_tests(full_path, tests, f)
            else:
                if f == "main.sk":
                    found = (parent, path)
                if f == "SKIP":
                    print("Skipping %s" % parent)
                    skip = True
        if found and not skip:
            tests.append(found)

def mkdir_safe(folder_name):
    try:
        os.mkdir(folder_name)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def run_command(args, name, verbose = False):
    try:
        #print("Running %s" % args)
        if verbose:
            subprocess.run(args, check=True)
        else:
            subprocess.run(args, check=True, stderr=subprocess.DEVNULL)
        return True
    except subprocess.CalledProcessError as e:
        print("%s failed" % name)
        return False
    except:
        print("%s failed" % name)
        return False

def getStd():
    return ["std2/*.sk", "std2/Json/*.sk"]

def run(silent, verbose, interpret, nostd, debug, test_name, source_folder, index, total):
    if not silent:
        print("--- Running %s - %d/%d - " % (test_name, total, index), end = "", flush=True)
    mkdir_safe("test_runs")
    target_folder = os.path.join("test_runs", test_name)
    mkdir_safe(target_folder)
    compiled_sikoc = os.path.join("..", "rust", "compiled_sikoc","sikoc")
    std = getStd()
    if nostd:
        std = []
    if not interpret and os.path.exists(compiled_sikoc):
        if not run_command(["./compiled_sikoc.sh", debug, verbose] + std + ["%s/*.sk" % source_folder, "-o", os.path.join(target_folder, test_name)], "compiled_sikoc", verbose = True):
            return False
    else:
        if not run_command(["./sikoc.sh"] + std + ["%s/*.sk" % source_folder, "-o", os.path.join(target_folder, test_name)], "normal sikoc", verbose = True):
            return False
    normal_output = os.path.join(target_folder, "normal")
    rc_output = os.path.join(target_folder, "rc")
    if not run_command(["rustc", "--edition=2018", os.path.join(target_folder, "%s_normal.rs" % test_name), "-o", normal_output], "normal rustc", verbose):
        return False
    return run_command([normal_output], "normal build")

test_source_name = "tests"
tests = []
verbose = ""
debug = ""
nostd = False
interpret = False
silent = False
collect_tests(test_source_name, tests, None)
if len(sys.argv) != 1:
    selected = set()
    for t in sys.argv[1:]:
        if t == "-v" or t == "-vv":
            verbose = t
            continue
        elif t == "-d":
            debug = "-d"
            continue
        elif t == "-i":
            interpret = True
        elif t == "-nostd":
            nostd = True
        elif t == "-s":
            silent = True
        else:
            selected.add(t)
    tests = list(filter(lambda test: test[0] in selected, tests))
total = len(tests)
success = 0
failure = 0
for (index, (name, path)) in enumerate(tests):
    start_time = time.time()
    result = run(silent, verbose, interpret, nostd, debug, name, path, index + 1, total)
    end_time = time.time()
    elapsed_time = end_time - start_time
    if not silent:
        print("{:.2f}s".format(elapsed_time))
    if result:
        success += 1
    else:
        failure += 1
if not silent:
    print("Success %d, failure %d" % (success, failure))
if failure > 0:
    print("Tests failed")
    sys.exit(1)