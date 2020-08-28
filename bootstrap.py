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

def processDir(path):
    source = ""
    if os.path.isdir(path):
        files = os.listdir(path)
        for f in files:
            full_path = os.path.join(path, f)
            if os.path.isdir(full_path):
                source += processDir(full_path)
            else:
                source += processFile(full_path)
    else:
        source += processFile(path)
    return source

def processSources(args):
    source = ""
    for arg in args:
        source += processDir(arg)
    return source

def mkdir_safe(folder_name):
    try:
        os.mkdir(folder_name)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def link_safe(src, dst):
    try:
        os.symlink(src, dst)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def compile_and_run(folder_name):
    source = processSources(["std2", "sikoc"])
    os.chdir(folder_name)
    f = open("sikoc.sk", "w")
    f.write(source)
    f.close()
    print("Starting stage 0")
    subprocess.call(["date"])
    subprocess.call(["./stage0"])
    subprocess.call(["cp", "sikoc_output.rs", "sikoc_output_stage_0.rs" ])
    subprocess.call(["date"])
    build_and_run(1)
    build_and_run(2)

def build_and_run(stage):
    print("Starting stage %s" % stage)
    subprocess.call(["date"])
    subprocess.call(["rustc", "sikoc_output.rs", "-o", "stage%s" % stage, "-O"])
    subprocess.call(["date"])
    subprocess.call(["./stage%s" % stage])
    subprocess.call(["cp", "sikoc_output.rs", "sikoc_output_stage_%s.rs" % stage])

folder_name = "bootstrap"
mkdir_safe(folder_name)
subprocess.call(["./siko", "-s", "std", "sikoc", "-c", os.path.join(folder_name, "source.rs")])
link_safe(os.path.join(os.getcwd(), "rt", "main.rs"), os.path.join(folder_name, "main.rs"))
subprocess.call(["rustc", "--edition=2018", os.path.join(folder_name, "main.rs"), "-o", os.path.join(folder_name, "stage0"), "--crate-name", "sikoc_rust", "-O"])
compile_and_run(folder_name)