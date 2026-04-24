"""
Python-only example — server side.

Requirements:
    pip install eruspy          # or: maturin develop (from eruspy-python/)

Run:
    python server.py

Then run the client in another terminal:
    python client.py
"""

import eruspy

HOST    = "0.0.0.0:3000"
STORAGE = "./storage"

print(f"[ python server ] http://{HOST}  storage: {STORAGE}")
print("Press Ctrl+C to stop.\n")

# run_server blocks until Ctrl+C
eruspy.run_server(STORAGE, True, HOST)
