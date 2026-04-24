"""
Python-only example — server side.

Run:
    python server.py

Then run the client in another terminal:
    python client.py
"""

import time
import eruspy

HOST    = "0.0.0.0:3000"
STORAGE = "./storage"

# run_server spawns a background Rust thread and returns immediately.
eruspy.run_server(STORAGE, True, HOST)
print(f"[ python server ] http://{HOST}  storage: {STORAGE}")
print("Press Ctrl+C to stop.\n")

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    print("stopped.")
