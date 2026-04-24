"""
Mixed example — Python server / Rust client.

Run this file first:
    python server.py

Then in another terminal, run the Rust client:
    cargo run --bin client
"""

import time
import eruspy

HOST    = "0.0.0.0:3000"
STORAGE = "./storage"

eruspy.run_server(STORAGE, True, HOST)
print(f"[ python server ] http://{HOST}  storage: {STORAGE}")
print("Waiting for Rust client... Press Ctrl+C to stop.\n")

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    print("stopped.")
