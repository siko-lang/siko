#!/usr/bin/python3

import sys
import os
import errno
import subprocess

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
        for f in files:
            full_path = os.path.join(path, f)
            if os.path.isdir(full_path):
                collect_tests(full_path, tests, f)
            else:
                if f == "main.sk":
                    tests.append((parent, path))

def mkdir_safe(folder_name):
    try:
        os.mkdir(folder_name)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def run(test_name, source_folder, index, total):
    print("--- Running %s - %d/%d" % (test_name, total, index))
    mkdir_safe("sikoc_test_runs")
    target_folder = os.path.join("sikoc_test_runs", test_name)
    mkdir_safe(target_folder)
    subprocess.call(["./siko.py", target_folder, "std2", source_folder])

test_source_name = "sikoc_tests"
tests = []
collect_tests(test_source_name, tests, None)
if len(sys.argv) != 1:
    selected = set()
    for t in sys.argv[1:]:
        selected.add(t)
    tests = list(filter(lambda test: test[0] in selected, tests))
total = len(tests)
for (index, (name, path)) in enumerate(tests):
    run(name, path, index + 1, total)