#!/usr/bin/env python3

import os
import shutil
import tempfile

TEST_FILE = "rpc_test.txt"

print("=" * 60)
print("440 RPC Filesystem Integration Test")
print("=" * 60)

# --------------------------------------------------
# Create test file
# --------------------------------------------------

print("[1] Creating test file...")

with open(TEST_FILE, "w") as f:
    f.write("Hello RPC!\n")
    f.write("Second line\n")

assert os.path.exists(TEST_FILE)

# --------------------------------------------------
# Read
# --------------------------------------------------

print("[2] Reading file...")

with open(TEST_FILE, "r") as f:
    data = f.read()

assert "Hello RPC!" in data
print(data)

# --------------------------------------------------
# Append
# --------------------------------------------------

print("[3] Appending...")

with open(TEST_FILE, "a") as f:
    f.write("Third line\n")

with open(TEST_FILE) as f:
    data = f.read()

assert "Third line" in data

# --------------------------------------------------
# Seek
# --------------------------------------------------

print("[4] Testing seek...")

with open(TEST_FILE) as f:
    f.seek(6)
    data = f.read(3)

assert data == "RPC"

# --------------------------------------------------
# Stat
# --------------------------------------------------

print("[5] Testing stat...")

st = os.stat(TEST_FILE)

assert st.st_size > 0

print("Size:", st.st_size)

# --------------------------------------------------
# Directory Listing
# --------------------------------------------------

print("[6] Listing directory...")

entries = os.listdir(".")

assert TEST_FILE in entries

print(entries)

# --------------------------------------------------
# Nested Directory Test
# --------------------------------------------------

print("[7] Directory tree...")

tmp = tempfile.mkdtemp()

os.mkdir(os.path.join(tmp, "a"))

os.mkdir(os.path.join(tmp, "a", "b"))

open(os.path.join(tmp, "a", "b", "file.txt"), "w").close()

for root, dirs, files in os.walk(tmp):
    print(root)
    print("dirs :", dirs)
    print("files:", files)

shutil.rmtree(tmp)

# --------------------------------------------------
# Delete
# --------------------------------------------------

print("[8] Testing unlink...")

os.unlink(TEST_FILE)

assert not os.path.exists(TEST_FILE)

print("Deleted.")

print()
print("=" * 60)
print("ALL TESTS PASSED")
print("=" * 60)