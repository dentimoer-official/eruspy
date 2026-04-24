//! Server-side file transfer handlers for actix-web.
//!
//! # Quick start
//!
//! Add eruspy with the `server` feature to your `Cargo.toml`:
//!
//! ```toml
//! eruspy = { version = "0.1", features = ["server"] }
//! ```
//!
//! Then mount [`transfer_scope`] anywhere inside your [`actix_web::App`]:
//!
//! ```rust,ignore
//! use actix_web::{web, App, HttpServer};
//! use eruspy::server::transfer_scope;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             // All transfer routes live under /files
//!             .service(
//!                 web::scope("/files")
//!                     .service(transfer_scope("./storage", true))
//!             )
//!     })
//!     .bind("0.0.0.0:3000")?
//!     .run()
//!     .await
//! }
//! ```
//!
//! Routes added by the scope (relative to its mount point):
//!
//! | Method | Path          | Description                              |
//! |--------|---------------|------------------------------------------|
//! | POST   | `/up`         | Upload a file (raw bytes body)           |
//! | GET    | `/down`       | Download a file                          |
//! | POST   | `/fup`        | Upload a folder (zip bytes body)         |
//! | GET    | `/fdown`      | Download a folder (zip response)         |
//! | GET    | `/list`       | List a directory (if `allow_list=true`)  |
//!
//! All routes accept a `?path=<relative-path>` query parameter.

use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::util::{safe_join, unzip_to, zip_dir};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Runtime configuration injected into every handler via [`actix_web::web::Data`].
pub struct TransferConfig {
    /// Root directory on the server where files are stored.
    pub root: PathBuf,
    /// Whether clients may call the `/list` endpoint.
    pub allow_list: bool,
}

// ---------------------------------------------------------------------------
// Shared query extractor
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PathQuery {
    path: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /up?path=<rel>` — upload a file (raw bytes body).
async fn upload_file(
    cfg: web::Data<TransferConfig>,
    query: web::Query<PathQuery>,
    body: web::Bytes,
) -> HttpResponse {
    let Some(full) = safe_join(&cfg.root, &query.path) else {
        return HttpResponse::BadRequest().json("invalid path");
    };
    // Check that the parent directory already exists — we do not create it.
    if let Some(parent) = full.parent() {
        if !parent.exists() {
            return HttpResponse::BadRequest().json("parent directory does not exist");
        }
    }
    match web::block(move || std::fs::write(&full, &*body)).await {
        Ok(Ok(_)) => HttpResponse::Ok().json("ok"),
        _ => HttpResponse::InternalServerError().json("write failed"),
    }
}

/// `GET /down?path=<rel>` — download a file (raw bytes response).
async fn download_file(
    cfg: web::Data<TransferConfig>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
    let Some(full) = safe_join(&cfg.root, &query.path) else {
        return HttpResponse::BadRequest().json("invalid path");
    };
    match web::block(move || std::fs::read(&full)).await {
        Ok(Ok(bytes)) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(bytes),
        _ => HttpResponse::NotFound().json("file not found"),
    }
}

/// `POST /fup?path=<rel>` — upload a folder (body must be a zip archive).
async fn upload_dir(
    cfg: web::Data<TransferConfig>,
    query: web::Query<PathQuery>,
    body: web::Bytes,
) -> HttpResponse {
    let Some(full) = safe_join(&cfg.root, &query.path) else {
        return HttpResponse::BadRequest().json("invalid path");
    };
    // Check that the parent directory already exists — we do not create it.
    if let Some(parent) = full.parent() {
        if !parent.exists() {
            return HttpResponse::BadRequest().json("parent directory does not exist");
        }
    }
    match web::block(move || unzip_to(&body, &full)).await {
        Ok(Ok(_)) => HttpResponse::Ok().json("ok"),
        _ => HttpResponse::InternalServerError().json("unzip failed"),
    }
}

/// `GET /fdown?path=<rel>` — download a folder as a zip archive.
async fn download_dir(
    cfg: web::Data<TransferConfig>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
    let Some(full) = safe_join(&cfg.root, &query.path) else {
        return HttpResponse::BadRequest().json("invalid path");
    };
    match web::block(move || zip_dir(&full)).await {
        Ok(Ok(bytes)) => HttpResponse::Ok()
            .content_type("application/zip")
            .body(bytes),
        _ => HttpResponse::InternalServerError().json("zip failed"),
    }
}

/// A single entry returned by the `/list` endpoint.
#[derive(Serialize)]
pub struct FileEntry {
    /// File or directory name (not a full path).
    pub name: String,
    /// `true` if this entry is a directory.
    pub is_dir: bool,
    /// Size in bytes; `0` for directories.
    pub size: u64,
}

#[derive(Serialize)]
struct ListResponse {
    /// The relative path that was listed (mirrors the `?path=` query).
    path: String,
    entries: Vec<FileEntry>,
}

/// `GET /list?path=<rel>` — list directory contents.
///
/// Returns `403 Forbidden` when [`TransferConfig::allow_list`] is `false`.
async fn list_dir(
    cfg: web::Data<TransferConfig>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
    if !cfg.allow_list {
        return HttpResponse::Forbidden().json("directory listing is disabled on this server");
    }

    let Some(full) = safe_join(&cfg.root, &query.path) else {
        return HttpResponse::BadRequest().json("invalid path");
    };

    let path_str = query.path.clone();

    match web::block(move || -> std::io::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&full)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: meta.is_dir(),
                size: if meta.is_file() { meta.len() } else { 0 },
            });
        }
        // Sort: directories first, then alphabetically within each group.
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
        Ok(entries)
    })
    .await
    {
        Ok(Ok(entries)) => HttpResponse::Ok().json(ListResponse {
            path: path_str,
            entries,
        }),
        _ => HttpResponse::NotFound().json("path not found or not a directory"),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build an actix-web [`Scope`] that handles all file transfer routes.
///
/// # Parameters
///
/// - `root`       — directory on the server where files are stored.
///                  Created at startup if it does not exist.
/// - `allow_list` — whether clients may call `GET /list` to browse the server.
///
/// # Example
///
/// ```rust,ignore
/// App::new().service(
///     web::scope("/transfer")
///         .service(transfer_scope("./storage", true))
/// )
/// ```
pub fn transfer_scope(root: impl Into<PathBuf>, allow_list: bool) -> Scope {
    let root = root.into();
    // Best-effort: create the root directory so the server starts cleanly.
    let _ = std::fs::create_dir_all(&root);

    let cfg = web::Data::new(TransferConfig { root, allow_list });

    web::scope("")
        .app_data(cfg)
        .route("/up",    web::post().to(upload_file))
        .route("/down",  web::get().to(download_file))
        .route("/fup",   web::post().to(upload_dir))
        .route("/fdown", web::get().to(download_dir))
        .route("/list",  web::get().to(list_dir))
}
