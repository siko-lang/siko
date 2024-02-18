#!/usr/bin/python3

import os
import subprocess

success = 0
failure = 0

def test(entry):
    print("- %s" % entry)
    global success, failure
    input_path = os.path.join("test", entry, "main.sk")
    output_path = os.path.join("test", entry, "main.rs")
    rust_output_path = os.path.join("test", entry, "main.bin")
    r = subprocess.run(["./siko.py", input_path, "-o", output_path])
    if r.returncode != 0:
        failure += 1
        return
    subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        failure += 1
        return
    success += 1

for entry in os.listdir("./test"):
    test(entry)
print("Success %s/%s - %.2f%%" % (success, success + failure, success/(success+failure)*100))
