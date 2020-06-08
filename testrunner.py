#!/usr/bin/python

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

def prepare(folder_name):
    try:
        os.mkdir(folder_name)
        os.symlink(os.path.join(os.getcwd(), "siko"), os.path.join(folder_name, "siko"))
        os.symlink(os.path.join(os.getcwd(), "sikoc"), os.path.join(folder_name, "sikoc"))
        os.symlink(os.path.join(os.getcwd(), "std"), os.path.join(folder_name, "std"))
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def run(test_name, source_folder):
    print "--- Running %s" % test_name
    target_folder = os.path.join("sikoc_test_runs", test_name)
    subprocess.call(["./siko.py", target_folder, "std2", source_folder])

test_source_name = sys.argv[1]
tests = []
collect_tests(test_source_name, tests, None)
if len(sys.argv) == 2:
    for (name, path) in tests:
        run(name, path)
else:
    selected = set()
    for t in sys.argv[2:]:
        selected.add(t)
    for (name, path) in tests:
        if name in selected:
            run(name, path)