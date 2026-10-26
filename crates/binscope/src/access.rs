//! Which files the server may read.
//!
//! By default any readable file can be inspected. Set `BINSCOPE_ROOT` to a
//! directory to confine the server to it (symlinks are resolved first, so they
//! cannot escape). Files above [`MAX_FILE_BYTES`] are refused rather than
//! loaded into memory.

use std::path::{Path, PathBuf};

/// Largest file binscope will load (512 MiB).
pub const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AccessError {
    #[error("{0} does not exist or cannot be read")]
    NotFound(String),
    #[error("{path} is outside BINSCOPE_ROOT ({root})")]
    OutsideRoot { path: String, root: String },
    #[error("{0} is a directory; pass a file")]
    Directory(String),
    #[error("{path} is {size} bytes, over the {limit}-byte limit")]
    TooLarge { path: String, size: u64, limit: u64 },
}

/// Resolve `requested` and check it against `root` and the size limit.
pub fn resolve(requested: &str, root: Option<&Path>) -> Result<PathBuf, AccessError> {
    let path = Path::new(requested.trim());
    let real = path
        .canonicalize()
        .map_err(|_| AccessError::NotFound(requested.to_string()))?;
    if let Some(root) = root {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if !real.starts_with(&root) {
            return Err(AccessError::OutsideRoot {
                path: real.display().to_string(),
                root: root.display().to_string(),
            });
        }
    }
    let meta = std::fs::metadata(&real).map_err(|_| AccessError::NotFound(requested.to_string()))?;
    if meta.is_dir() {
        return Err(AccessError::Directory(real.display().to_string()));
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(AccessError::TooLarge {
            path: real.display().to_string(),
            size: meta.len(),
            limit: MAX_FILE_BYTES,
        });
    }
    Ok(real)
}

/// The root configured through the environment, if any.
pub fn root_from_env() -> Option<PathBuf> {
    std::env::var_os("BINSCOPE_ROOT").filter(|v| !v.is_empty()).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!("binscope-access-{}", std::process::id()));
        std::fs::create_dir_all(d.join("inside")).unwrap();
        std::fs::write(d.join("inside/a.bin"), b"x").unwrap();
        std::fs::write(d.join("outside.bin"), b"y").unwrap();
        d
    }

    #[test]
    fn allows_files_under_root_and_refuses_escapes() {
        let d = tmp();
        let root = d.join("inside");
        assert!(resolve(d.join("inside/a.bin").to_str().unwrap(), Some(&root)).is_ok());
        let escape = d.join("inside/../outside.bin");
        assert!(matches!(
            resolve(escape.to_str().unwrap(), Some(&root)),
            Err(AccessError::OutsideRoot { .. })
        ));
    }

    #[test]
    fn rejects_missing_files_and_directories() {
        let d = tmp();
        assert!(matches!(resolve("/definitely/not/here", None), Err(AccessError::NotFound(_))));
        assert!(matches!(resolve(d.to_str().unwrap(), None), Err(AccessError::Directory(_))));
    }
}
