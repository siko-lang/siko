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
    for arg in sys.argv[1:]:
        source += processDir(arg)
    return source

source = processSources(sys.argv[1:])

try:
    os.mkdir("sikocwd")
except OSError as e:
    if e.errno != errno.EEXIST:
        raise
os.chdir("sikocwd")
f = open("sikoc.sk", "w")
f.write(source)
f.close()

subprocess.call(["./siko", "sikoc"])