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

def processDir(path):
    source = ""
    if os.path.isdir(path):
        files = os.listdir(path)
        for f in files:
            if os.path.isdir(f):
                source += processDir(os.path.join(path, f))
            else:
                source += processFile(os.path.join(path, f))
    else:
        source += processFile(path)
    return source

def processSources(args):
    source = ""
    for arg in args:
        source += processDir(arg)
    return source

def prepare(folder_name):
    try:
        os.mkdir(folder_name)
        os.symlink(os.path.join(os.getcwd(), "siko"), os.path.join(folder_name, "siko"))
        os.symlink(os.path.join(os.getcwd(), "sikoc"), os.path.join(folder_name, "sikoc"))
        os.symlink(os.path.join(os.getcwd(), "std"), os.path.join(folder_name, "std"))
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def compile_and_run(folder_name):
    source = processSources(sys.argv[2:])
    prepare(folder_name)
    os.chdir(folder_name)
    f = open("sikoc.sk", "w")
    f.write(source)
    f.close()

    subprocess.call(["./siko", "sikoc"])
    subprocess.call(["rustc", "sikoc_output.rs", "-o", "rust_program"])
    subprocess.call(["./rust_program"])

folder_name = sys.argv[1]
compile_and_run(folder_name)