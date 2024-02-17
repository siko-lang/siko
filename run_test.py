#!/usr/bin/python3

import os
import subprocess

def test(entry):
    print("Testing %s" % entry)
    input_path = os.path.join("test", entry, "main.sk")
    output_path = os.path.join("test", entry, "main.rs")
    rust_output_path = os.path.join("test", entry, "main.bin")
    subprocess.run(["./siko.py", input_path, "-o", output_path])
    subprocess.run(["rustc", output_path, "-o", rust_output_path])

for entry in os.listdir("./test"):
    test(entry)
