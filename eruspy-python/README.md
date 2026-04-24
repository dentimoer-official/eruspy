# eruspy-python

Python bindings for [eruspy](https://crates.io/crates/eruspy) — file and directory transfer over HTTP, built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).

## Installation

```bash
pip install eruspy
```

## Quick Start

### Client

```python
import eruspy

c = eruspy.EruspyClient("http://localhost:3000/transfer")

# Upload / download a file
c.upload("./file.txt", "file.txt")
c.download("file.txt", "./received.txt")

# Upload / download a directory
c.upload_dir("./my_folder", "my_folder")
c.download_dir("my_folder", "./restored")

# List a directory
entries = c.list("")           # "" = storage root
for e in entries:
    kind = "DIR" if e.is_dir else "FILE"
    print(f"[{kind}] {e.name}  ({e.size} bytes)")
```

### Server

`run_server` blocks the calling thread. Use `threading.Thread` to run it in the background:

```python
import threading
import eruspy

t = threading.Thread(
    target=eruspy.run_server,
    args=("./storage", True, "0.0.0.0:3000"),
    #      root dir    ^     ^
    #      allow_list ─┘     └─ bind address
    daemon=True,
)
t.start()
```

Or run it directly (blocks until Ctrl+C):

```python
import eruspy
eruspy.run_server("./storage", True, "0.0.0.0:3000")
```

## API Reference

### `EruspyClient(base_url)`

| Method | Description |
|--------|-------------|
| `upload(local, remote)` | Upload a file. Parent dir must exist on server. |
| `download(remote, local)` | Download a file. |
| `upload_dir(local, remote)` | Upload a directory (zipped). Parent dir must exist on server. |
| `download_dir(remote, local)` | Download a directory (unzipped locally). |
| `list(remote_path)` | List a directory. Returns `list[FileEntry]`. |

### `FileEntry`

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | File or directory name |
| `is_dir` | `bool` | `True` if directory |
| `size` | `int` | Size in bytes (`0` for directories) |

### `run_server(storage, allow_list, host)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `storage` | `str` | Root directory for stored files |
| `allow_list` | `bool` | Allow clients to call the `/list` endpoint |
| `host` | `str` | Bind address, e.g. `"0.0.0.0:3000"` |

## Building from Source

```bash
cd eruspy-python
python3 -m venv .venv
source .venv/bin/activate
pip install maturin
maturin develop          # dev install into current venv
maturin build --release  # produce a .whl for distribution
```

## License

MIT OR Apache-2.0
