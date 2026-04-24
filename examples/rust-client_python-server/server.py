"""
Mixed example — Python server / Rust client.

Requirements:
    pip install eruspy          # or: maturin develop (from eruspy-python/)

Run this file first:
    python server.py

Then in another terminal, run the Rust client:
    cargo run --bin client
"""

import eruspy

HOST    = "0.0.0.0:3000"
STORAGE = "./storage"

print(f"[ python server ] http://{HOST}  storage: {STORAGE}")
print("Waiting for Rust client... Press Ctrl+C to stop.\n")

eruspy.run_server(STORAGE, True, HOST)
