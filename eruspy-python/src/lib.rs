//! Python bindings for eruspy — file and directory transfer over HTTP.

// Use :: prefix to avoid ambiguity with the #[pymodule] fn named `eruspy`
use ::eruspy::client::EruspyClient as RustClient;
use ::eruspy::client::FileEntry as RustFileEntry;
use ::eruspy::server::transfer_scope;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// FileEntry
// ---------------------------------------------------------------------------

/// A single file or directory entry returned by :meth:`EruspyClient.list`.
#[pyclass(get_all, skip_from_py_object)]
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

impl From<RustFileEntry> for FileEntry {
    fn from(e: RustFileEntry) -> Self {
        FileEntry { name: e.name, is_dir: e.is_dir, size: e.size }
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
///     Full URL to the transfer scope, e.g. ``"http://localhost:3000/transfer"``.
#[pyclass]
struct EruspyClient(RustClient);

#[pymethods]
impl EruspyClient {
    #[new]
    fn new(base_url: &str) -> Self {
        EruspyClient(RustClient::new(base_url))
    }

    /// Upload a local file to the server.
    fn upload(&self, local: &str, remote: &str) -> PyResult<()> {
        self.0.upload(local, remote).map_err(PyRuntimeError::new_err)
    }

    /// Download a file from the server to a local path.
    fn download(&self, remote: &str, local: &str) -> PyResult<()> {
        self.0.download(remote, local).map_err(PyRuntimeError::new_err)
    }

    /// Upload a local directory (sent as zip). Parent dir must exist on server.
    fn upload_dir(&self, local: &str, remote: &str) -> PyResult<()> {
        self.0.upload_dir(local, remote).map_err(PyRuntimeError::new_err)
    }

    /// Download a directory from the server (extracted locally).
    fn download_dir(&self, remote: &str, local: &str) -> PyResult<()> {
        self.0.download_dir(remote, local).map_err(PyRuntimeError::new_err)
    }

    /// List a directory. Returns ``list[FileEntry]``, dirs first then alphabetical.
    fn list(&self, remote_path: &str) -> PyResult<Vec<FileEntry>> {
        self.0
            .list(remote_path)
            .map(|entries: Vec<RustFileEntry>| {
                entries.into_iter().map(FileEntry::from).collect::<Vec<FileEntry>>()
            })
            .map_err(PyRuntimeError::new_err)
    }
}

// ---------------------------------------------------------------------------
// run_server
// ---------------------------------------------------------------------------

/// Start a transfer server in a background Rust thread and return immediately.
///
/// The server keeps the process alive until stopped (Ctrl+C or process exit).
///
/// Parameters
/// ----------
/// storage : str
///     Root directory where uploaded files are stored.
///     Created automatically if it does not exist.
/// allow_list : bool
///     ``True`` — clients may call ``GET /transfer/list``.
///     ``False`` — returns **403 Forbidden**.
/// host : str
///     Bind address, e.g. ``"0.0.0.0:3000"`` or ``"127.0.0.1:8080"``.
///
/// Examples
/// --------
/// >>> import eruspy, time
/// >>> eruspy.run_server("./storage", True, "0.0.0.0:3000")
/// >>> # server is running in background; keep the process alive:
/// >>> try:
/// ...     while True: time.sleep(1)
/// ... except KeyboardInterrupt:
/// ...     pass
#[pyfunction]
fn run_server(storage: String, allow_list: bool, host: String) -> PyResult<()> {
    // Spawn a Rust-native thread — does not hold the Python GIL.
    std::thread::Builder::new()
        .name("eruspy-server".to_owned())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime failed to start");

            rt.block_on(async move {
                use actix_web::{web, App, HttpServer};

                println!("eruspy server  →  http://{host}  (storage: {storage})");

                HttpServer::new(move || {
                    App::new().service(
                        web::scope("/transfer")
                            .service(transfer_scope(storage.clone(), allow_list)),
                    )
                })
                .bind(&host)
                .expect("failed to bind address")
                .run()
                .await
                .expect("server error");
            });
        })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule]
fn eruspy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FileEntry>()?;
    m.add_class::<EruspyClient>()?;
    m.add_function(wrap_pyfunction!(run_server, m)?)?;
    Ok(())
}
