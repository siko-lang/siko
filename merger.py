#!/usr/bin/python

import sys

def processFile(file):
    f = open(file)
    lines = f.readlines()
    for line in lines:
        l = line.rstrip()
        print "%s" % l
    last = lines[-1]
    last = last.rstrip()
    if last != "":
        print "\n"

for arg in sys.argv[1:]:
    processFile(arg)