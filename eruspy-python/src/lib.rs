//! Python bindings for eruspy — file and directory transfer over HTTP.
//!
//! Built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).
//!
//! # Python usage
//!
//! ```python
//! import eruspy
//!
//! # --- Client ---
//! c = eruspy.EruspyClient("http://localhost:3000/transfer")
//! c.upload("./file.txt", "file.txt")
//! c.download("file.txt", "./received.txt")
//! c.upload_dir("./folder", "folder")
//! c.download_dir("folder", "./restored")
//!
//! entries = c.list("")
//! for e in entries:
//!     print(f"{'DIR' if e.is_dir else 'FILE'} {e.name} ({e.size} bytes)")
//!
//! # --- Server (blocking — run in a thread for background use) ---
//! import threading
//! t = threading.Thread(
//!     target=eruspy.run_server,
//!     args=("./storage", True, "0.0.0.0:3000"),
//!     daemon=True,
//! )
//! t.start()
//! ```

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// FileEntry
// ---------------------------------------------------------------------------

/// A single file or directory entry returned by :meth:`EruspyClient.list`.
///
/// Attributes
/// ----------
/// name : str
///     File or directory name (not a full path).
/// is_dir : bool
///     ``True`` if this entry is a directory.
/// size : int
///     Size in bytes; ``0`` for directories.
#[pyclass(get_all)]
#[derive(Clone)]
struct FileEntry {
    /// File or directory name (not a full path).
    name: String,
    /// ``True`` if this entry is a directory.
    is_dir: bool,
    /// Size in bytes; ``0`` for directories.
    size: u64,
}

#[pymethods]
impl FileEntry {
    fn __repr__(&self) -> String {
        format!(
            "FileEntry(name={:?}, is_dir={}, size={})",
            self.name, self.is_dir, self.size
        )
    }
}

impl From<eruspy::client::FileEntry> for FileEntry {
    fn from(e: eruspy::client::FileEntry) -> Self {
        FileEntry {
            name:   e.name,
            is_dir: e.is_dir,
            size:   e.size,
        }
    }
}

// ---------------------------------------------------------------------------
// EruspyClient
// ---------------------------------------------------------------------------

/// Synchronous client for an eruspy transfer server.
///
/// Parameters
/// ----------
/// base_url : str
///     Full URL to the transfer scope on the server, e.g.
///     ``"http://localhost:3000/transfer"`` or
///     ``"https://example.com/transfer"``.
///     Trailing slashes are stripped automatically.
///
/// Examples
/// --------
/// >>> c = EruspyClient("http://localhost:3000/transfer")
/// >>> c.upload("./hello.txt", "hello.txt")
/// >>> c.download("hello.txt", "./received.txt")
/// >>> entries = c.list("")
/// >>> for e in entries:
/// ...     print(e.name, e.size)
#[pyclass]
struct EruspyClient(eruspy::client::EruspyClient);

#[pymethods]
impl EruspyClient {
    /// Create a new client pointing at *base_url*.
    #[new]
    fn new(base_url: &str) -> Self {
        EruspyClient(eruspy::client::EruspyClient::new(base_url))
    }

    /// Upload a local file to the server.
    ///
    /// Parameters
    /// ----------
    /// local : str
    ///     Path to the file on this machine.
    /// remote : str
    ///     Relative path on the server (e.g. ``"data/file.txt"``).
    ///     The parent directory must already exist on the server.
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If the upload fails (network error, server error, etc.).
    fn upload(&self, local: &str, remote: &str) -> PyResult<()> {
        self.0
            .upload(local, remote)
            .map_err(PyRuntimeError::new_err)
    }

    /// Download a file from the server to a local path.
    ///
    /// Parameters
    /// ----------
    /// remote : str
    ///     Relative path on the server.
    /// local : str
    ///     Where to save the file on this machine.
    ///     Parent directories are created automatically.
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If the download fails.
    fn download(&self, remote: &str, local: &str) -> PyResult<()> {
        self.0
            .download(remote, local)
            .map_err(PyRuntimeError::new_err)
    }

    /// Upload a local directory to the server (sent as a zip archive).
    ///
    /// Parameters
    /// ----------
    /// local : str
    ///     Path to the directory on this machine.
    /// remote : str
    ///     Relative path on the server where the directory will be extracted.
    ///     The parent directory must already exist on the server.
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If the upload fails.
    fn upload_dir(&self, local: &str, remote: &str) -> PyResult<()> {
        self.0
            .upload_dir(local, remote)
            .map_err(PyRuntimeError::new_err)
    }

    /// Download a directory from the server (received as a zip archive).
    ///
    /// Parameters
    /// ----------
    /// remote : str
    ///     Relative path on the server.
    /// local : str
    ///     Where to extract the directory on this machine.
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If the download fails.
    fn download_dir(&self, remote: &str, local: &str) -> PyResult<()> {
        self.0
            .download_dir(remote, local)
            .map_err(PyRuntimeError::new_err)
    }

    /// List files and directories at *remote_path* on the server.
    ///
    /// Parameters
    /// ----------
    /// remote_path : str
    ///     Relative path to list. Pass ``""`` to list the storage root.
    ///
    /// Returns
    /// -------
    /// list[FileEntry]
    ///     Entries sorted: directories first, then alphabetically.
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If listing is disabled on the server or the path does not exist.
    fn list(&self, remote_path: &str) -> PyResult<Vec<FileEntry>> {
        self.0
            .list(remote_path)
            .map(|v| v.into_iter().map(FileEntry::from).collect())
            .map_err(PyRuntimeError::new_err)
    }
}

// ---------------------------------------------------------------------------
// run_server
// ---------------------------------------------------------------------------

/// Start a blocking eruspy transfer server.
///
/// This function **blocks** the calling thread until the server is stopped
/// (e.g. by Ctrl+C).  Run it in a :class:`threading.Thread` for background
/// use::
///
///     import threading, eruspy
///
///     t = threading.Thread(
///         target=eruspy.run_server,
///         args=("./storage", True, "0.0.0.0:3000"),
///         daemon=True,
///     )
///     t.start()
///
/// Parameters
/// ----------
/// storage : str
///     Root directory where uploaded files are stored.
///     Created automatically if it does not exist.
/// allow_list : bool
///     ``True`` — clients may call ``GET /transfer/list``.
///     ``False`` — that endpoint returns **403 Forbidden**.
/// host : str
///     Address to bind, e.g. ``"0.0.0.0:3000"`` or ``"127.0.0.1:8080"``.
///
/// Raises
/// ------
/// RuntimeError
///     If the server fails to bind or encounters a fatal error.
#[pyfunction]
fn run_server(py: Python<'_>, storage: String, allow_list: bool, host: String) -> PyResult<()> {
    // Release the GIL while the server runs so other Python threads can work.
    let result: Result<(), String> = py.allow_threads(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        rt.block_on(async move {
            use actix_web::{web, App, HttpServer};
            use eruspy::server::transfer_scope;

            println!("eruspy server  →  http://{host}  (storage: {storage})");

            HttpServer::new(move || {
                App::new().service(
                    web::scope("/transfer")
                        .service(transfer_scope(storage.clone(), allow_list)),
                )
            })
            .bind(&host)
            .map_err(|e| e.to_string())?
            .run()
            .await
            .map_err(|e| e.to_string())
        })
    });

    result.map_err(PyRuntimeError::new_err)
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

#[pymodule]
fn eruspy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FileEntry>()?;
    m.add_class::<EruspyClient>()?;
    m.add_function(wrap_pyfunction!(run_server, m)?)?;
    Ok(())
}
