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

def link_safe(src, dst):
    try:
        os.symlink(src, dst)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def processSources(args):
    source = ""
    for arg in args:
        source += processDir(arg)
    return source

def prepare(folder_name):
    try:
        os.mkdir(folder_name)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise
    try:
        os.symlink(os.path.join(os.getcwd(), "siko"), os.path.join(folder_name, "siko"))
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise
    try:
        os.symlink(os.path.join(os.getcwd(), "sikoc"), os.path.join(folder_name, "sikoc"))
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise
    try:
        os.symlink(os.path.join(os.getcwd(), "std"), os.path.join(folder_name, "std"))
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

def compile_and_run(folder_name):
    prepare(folder_name)
    source = processSources(sys.argv[2:])
    f = open(os.path.join(folder_name, "sikoc.sk"), "w")
    f.write(source)
    f.close()

    compiled = os.path.join("compiled_sikoc", "sikoc_rust")
    if os.path.exists(compiled):
        cc = os.path.join(folder_name, "sikoc_rust")
        link_safe(os.path.join("..", "..", compiled), cc)
        os.chdir(folder_name)
        #print("Running compiled in %s" % folder_name)
        subprocess.call(["./sikoc_rust"])
    else:
        os.chdir(folder_name)
        #print("Running interpreter")
        subprocess.check_call(["./siko", "sikoc"])

    subprocess.check_call(["rustfmt", "sikoc_output.rs"])
    #if os.path.exists("./sikoc_output_normal.rs"):
    #    subprocess.call(["rustc", "sikoc_output_normal.rs", "-o", "rust_program_normal"])
    subprocess.check_call(["rustc", "sikoc_output.rs", "-o", "rust_program"])
    subprocess.check_call(["./rust_program"])

folder_name = sys.argv[1]
compile_and_run(folder_name)