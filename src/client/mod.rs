//! Client for interacting with an eruspy transfer server.
//!
//! # Quick start
//!
//! Add eruspy with the `client` feature to your `Cargo.toml`:
//!
//! ```toml
//! eruspy = { version = "0.1", features = ["client"] }
//! ```
//!
//! Then create an [`EruspyClient`] and call its methods:
//!
//! ```rust,ignore
//! use eruspy::client::EruspyClient;
//!
//! let c = EruspyClient::new("http://localhost:3000/transfer");
//!
//! // Upload / download a file
//! c.upload("./hello.txt", "greetings/hello.txt").unwrap();
//! c.download("greetings/hello.txt", "./received.txt").unwrap();
//!
//! // Upload / download a whole directory
//! c.upload_dir("./my_folder", "backups/my_folder").unwrap();
//! c.download_dir("backups/my_folder", "./restored").unwrap();
//!
//! // List what is on the server (requires allow_list=true on the server)
//! let entries = c.list("greetings").unwrap();
//! for e in &entries {
//!     println!("{} {}", if e.is_dir { "DIR" } else { "   " }, e.name);
//! }
//! ```

use serde::Deserialize;
use std::path::Path;

use crate::util::{unzip_to, zip_dir};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single file or directory entry returned by [`EruspyClient::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    /// File or directory name (not a full path).
    pub name: String,
    /// `true` if this entry is a directory.
    pub is_dir: bool,
    /// Size in bytes; `0` for directories.
    pub size: u64,
}

#[derive(Deserialize)]
struct ListResponse {
    #[allow(dead_code)]
    path: String,
    entries: Vec<FileEntry>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Synchronous client for an eruspy transfer server.
///
/// Works with any base URL — local (`http://localhost:3000/transfer`) or
/// remote (`https://example.com/transfer`).
///
/// All methods return `Ok(())` / `Ok(data)` on success and `Err(String)` with
/// a human-readable message on failure.
pub struct EruspyClient {
    /// Base URL of the transfer scope, e.g. `"http://localhost:3000/transfer"`.
    base_url: String,
    client: reqwest::blocking::Client,
}

impl EruspyClient {
    /// Create a new client pointing at `base_url`.
    ///
    /// `base_url` should be the full URL to the transfer scope mounted on the
    /// server, e.g. `"http://localhost:3000/transfer"` or
    /// `"https://example.com/files"`. Trailing slashes are stripped
    /// automatically.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Build a full URL for the given endpoint and remote path.
    fn url(&self, endpoint: &str, path: &str) -> String {
        format!("{}/{}?path={}", self.base_url, endpoint, path)
    }

    // -----------------------------------------------------------------------
    // File transfer
    // -----------------------------------------------------------------------

    /// Upload a local file to the server.
    ///
    /// - `local`  — path to the file on this machine.
    /// - `remote` — relative path on the server (e.g. `"data/hello.txt"`).
    pub fn upload(&self, local: impl AsRef<Path>, remote: &str) -> Result<(), String> {
        let bytes = std::fs::read(local.as_ref())
            .map_err(|e| format!("read local file: {e}"))?;

        let resp = self
            .client
            .post(self.url("up", remote))
            .body(bytes)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("server returned {}", resp.status()))
        }
    }

    /// Download a file from the server to a local path.
    ///
    /// - `remote` — relative path on the server.
    /// - `local`  — where to save the file on this machine.
    ///              Parent directories are created if they do not exist.
    pub fn download(&self, remote: &str, local: impl AsRef<Path>) -> Result<(), String> {
        let resp = self
            .client
            .get(self.url("down", remote))
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("server returned {}", resp.status()));
        }

        let bytes = resp.bytes().map_err(|e| format!("read response body: {e}"))?;
        let local = local.as_ref();

        if let Some(p) = local.parent() {
            std::fs::create_dir_all(p).map_err(|e| format!("create parent dirs: {e}"))?;
        }
        std::fs::write(local, &bytes).map_err(|e| format!("write file: {e}"))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Directory transfer
    // -----------------------------------------------------------------------

    /// Upload a local directory to the server (sent as a zip archive).
    ///
    /// - `local`  — path to the directory on this machine.
    /// - `remote` — relative path on the server where the directory will be
    ///              extracted.
    pub fn upload_dir(&self, local: impl AsRef<Path>, remote: &str) -> Result<(), String> {
        let bytes =
            zip_dir(local.as_ref()).map_err(|e| format!("zip local directory: {e}"))?;

        let resp = self
            .client
            .post(self.url("fup", remote))
            .body(bytes)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("server returned {}", resp.status()))
        }
    }

    /// Download a directory from the server to a local path (received as a zip
    /// archive that is extracted automatically).
    ///
    /// - `remote` — relative path on the server.
    /// - `local`  — where to extract the directory on this machine.
    pub fn download_dir(&self, remote: &str, local: impl AsRef<Path>) -> Result<(), String> {
        let resp = self
            .client
            .get(self.url("fdown", remote))
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("server returned {}", resp.status()));
        }

        let bytes = resp.bytes().map_err(|e| format!("read response body: {e}"))?;
        unzip_to(&bytes, local.as_ref()).map_err(|e| format!("unzip: {e}"))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Directory listing
    // -----------------------------------------------------------------------

    /// List files and directories at `remote_path` on the server.
    ///
    /// Returns [`Err`] if the server has listing disabled (`allow_list=false`)
    /// or if the path does not exist.
    ///
    /// Entries are sorted: directories first, then alphabetically.
    pub fn list(&self, remote_path: &str) -> Result<Vec<FileEntry>, String> {
        let resp = self
            .client
            .get(self.url("list", remote_path))
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(format!("server returned {status}: {body}"));
        }

        let parsed: ListResponse = resp
            .json()
            .map_err(|e| format!("parse response: {e}"))?;

        Ok(parsed.entries)
    }
}
