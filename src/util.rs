//! Internal utilities shared by the server and client modules.

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

/// Join `root` with `rel`, rejecting any path-traversal components (`..`, absolute roots).
///
/// Returns `None` if `rel` contains a component that would escape `root`.
pub fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let mut result = root.to_path_buf();
    for comp in Path::new(rel.trim_start_matches('/')).components() {
        match comp {
            Component::Normal(p) => result.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(result)
}

/// Compress the directory at `src` into a zip byte vector.
///
/// All entries are stored with paths relative to `src`.
pub fn zip_dir(src: &Path) -> std::io::Result<Vec<u8>> {
    use zip::write::SimpleFileOptions;

    let buf = std::io::Cursor::new(Vec::new());
    let mut zw = zip::ZipWriter::new(buf);
    let opts = SimpleFileOptions::default();
    let mut stack = vec![src.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            let rel = path
                .strip_prefix(src)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");

            if path.is_dir() {
                zw.add_directory(format!("{}/", rel), opts)?;
                stack.push(path);
            } else {
                zw.start_file(&rel, opts)?;
                zw.write_all(&std::fs::read(&path)?)?;
            }
        }
    }

    Ok(zw.finish()?.into_inner())
}

/// Extract a zip byte slice into `dest`, creating the directory if needed.
pub fn unzip_to(data: &[u8], dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    for i in 0..archive.len() {
        let mut f = archive
            .by_index(i)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let out = dest.join(f.name());

        if f.name().ends_with('/') {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            std::fs::write(&out, &buf)?;
        }
    }

    Ok(())
}
