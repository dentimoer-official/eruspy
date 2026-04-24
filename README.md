# eruspy

A minimal Rust library for transferring files and directories over HTTP.

Drop it into any [actix-web](https://actix.rs/) server with one line, and talk
to it from any machine using the built-in blocking client — or from plain HTTP
calls if you prefer.

---

## Features

| Feature | What it enables |
|---------|-----------------|
| `server` | actix-web route handlers — mount a transfer scope into your app |
| `client` | `EruspyClient` — a blocking HTTP client for upload / download / list |

Both features are independent. Enable only what you need.

---

## Installation

```toml
# server side
eruspy = { version = "0.1", features = ["server"] }

# client side
eruspy = { version = "0.1", features = ["client"] }

# both (e.g. for examples or integration tests)
eruspy = { version = "0.1", features = ["server", "client"] }
```

---

## Server

### Mounting the transfer scope

Call `transfer_scope(root, allow_list)` and mount it anywhere inside your
`App::new()` chain. That is the only change you need to make to an existing
actix-web application.

```rust
use actix_web::{web, App, HttpServer};
use eruspy::server::transfer_scope;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // your existing routes …
            .service(
                web::scope("/transfer")
                    .service(transfer_scope("./storage", true))
                //                          ─────────  ────
                //                          root dir   allow /list endpoint
            )
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
```

### Parameters

| Parameter    | Type   | Description |
|--------------|--------|-------------|
| `root`       | `impl Into<PathBuf>` | Directory where files are stored. Created automatically on startup. |
| `allow_list` | `bool` | `true` — clients may call `GET /list`. `false` — `GET /list` returns **403**. |

### Routes

All routes are relative to the scope prefix (e.g. `/transfer`).
Every route accepts a `?path=<relative-path>` query parameter.

| Method | Path     | Body         | Description |
|--------|----------|--------------|-------------|
| `POST` | `/up`    | raw bytes    | Upload a file. Parent directory must already exist. |
| `GET`  | `/down`  | —            | Download a file. |
| `POST` | `/fup`   | zip archive  | Upload a directory. Parent directory must already exist. |
| `GET`  | `/fdown` | —            | Download a directory as a zip archive. |
| `GET`  | `/list`  | —            | List a directory. Returns `403` if `allow_list` is `false`. |

> **Note:** eruspy does **not** create missing parent directories automatically.
> Upload to `data/file.txt` only works if `data/` already exists under the
> root. Use `/fup` first to create a directory, or upload directly to the root.

### List response format

```json
{
  "path": "some/dir",
  "entries": [
    { "name": "folder",   "is_dir": true,  "size": 0    },
    { "name": "file.txt", "is_dir": false, "size": 1024 }
  ]
}
```

Entries are sorted: directories first, then alphabetically within each group.

---

## Client

`EruspyClient` is a synchronous (blocking) client. Point it at any eruspy
server — local or remote.

```rust
use eruspy::client::EruspyClient;

let c = EruspyClient::new("http://localhost:3000/transfer");
```

### Upload a file

```rust
c.upload("./local/file.txt", "remote/file.txt")?;
```

### Download a file

```rust
c.download("remote/file.txt", "./local/received.txt")?;
```

### Upload a directory

The directory is compressed to a zip archive and extracted on the server.

```rust
c.upload_dir("./local/folder", "remote/folder")?;
```

### Download a directory

The server compresses the directory and the client extracts it locally.

```rust
c.download_dir("remote/folder", "./local/restored")?;
```

### List a directory

```rust
let entries = c.list("remote/folder")?;

for e in entries {
    let kind = if e.is_dir { "DIR " } else { "FILE" };
    println!("[{kind}] {} ({} bytes)", e.name, e.size);
}
```

Returns `Err` if the server has `allow_list = false` or the path does not exist.

### FileEntry fields

| Field    | Type   | Description |
|----------|--------|-------------|
| `name`   | `String` | File or directory name (not a full path) |
| `is_dir` | `bool`   | `true` if this entry is a directory |
| `size`   | `u64`    | Size in bytes; `0` for directories |

### Pointing at a different server

`new()` accepts any base URL. Trailing slashes are stripped automatically.

```rust
// local
let c = EruspyClient::new("http://localhost:3000/transfer");

// LAN
let c = EruspyClient::new("http://192.168.1.10:3000/transfer");

// remote
let c = EruspyClient::new("https://example.com/transfer");
```

---

## Error handling

All client methods return `Result<_, String>` with a human-readable message on
failure. Wrap them in your own error type or use `.expect()` / `?` as needed.

```rust
match c.upload("./file.txt", "file.txt") {
    Ok(())   => println!("uploaded"),
    Err(msg) => eprintln!("upload failed: {msg}"),
}
```

---

## Path rules

- Paths are **always relative** to the server's root directory.
- Path traversal (`..`, absolute roots) is rejected with `400 Bad Request`.
- Slashes at the start of a path are stripped (`/file.txt` == `file.txt`).

---

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.
