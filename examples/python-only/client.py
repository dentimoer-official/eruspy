"""
Python-only example — client side.

Requirements:
    pip install eruspy          # or: maturin develop (from eruspy-python/)

Run (server must be running first):
    python client.py

Override the server address:
    BASE_URL=http://192.168.1.10:3000/transfer python client.py
"""

import os
import shutil
import eruspy

BASE_URL = os.environ.get("BASE_URL", "http://localhost:3000/transfer")

print(f"[ python client ] connecting to {BASE_URL}\n")

c = eruspy.EruspyClient(BASE_URL)

# --- Upload a file ---
with open("hello.txt", "w") as f:
    f.write("Hello from Python client!\n")

c.upload("hello.txt", "hello.txt")
print("  uploaded   hello.txt")

# --- List root ---
entries = c.list("")
print(f"  list root  ({len(entries)} entries)")
for e in entries:
    kind = "DIR " if e.is_dir else "FILE"
    print(f"    [{kind}] {e.name}  ({e.size} bytes)")

# --- Download back ---
c.download("hello.txt", "received.txt")
with open("received.txt") as f:
    content = f.read().strip()
print(f"  downloaded hello.txt → received.txt")
print(f"  content: {content!r}")

# --- Upload a directory ---
os.makedirs("mydir", exist_ok=True)
with open("mydir/a.txt", "w") as f:
    f.write("alpha")
with open("mydir/b.txt", "w") as f:
    f.write("beta")

c.upload_dir("mydir", "mydir")
print("  uploaded   mydir/")

# --- Download directory back ---
shutil.rmtree("mydir_restored", ignore_errors=True)
c.download_dir("mydir", "mydir_restored")
print("  downloaded mydir/ → mydir_restored/")
for name in sorted(os.listdir("mydir_restored")):
    print(f"    {name}")

# --- Cleanup local temp files ---
for path in ["hello.txt", "received.txt", "mydir", "mydir_restored"]:
    shutil.rmtree(path, ignore_errors=True)
    try:
        os.remove(path)
    except FileNotFoundError:
        pass

print("\ndone.")
