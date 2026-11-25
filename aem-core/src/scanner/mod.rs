pub mod directories;

use crate::models::{Category, FileEntry};
use thiserror::Error;

/// Error during scanning.
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
}

/// Scan a directory tree and return all files with metadata.
pub fn scan_directory(root: &std::path::Path) -> Result<Vec<FileEntry>, ScanError> {
    if !root.exists() {
        return Err(ScanError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", root.display()),
        )));
    }
    directories::scan_tree(root)
}

/// Scan a directory tree and return files filtered by categories.
pub fn scan_directory_filtered(root: &std::path::Path, categories: &[Category]) -> Result<Vec<FileEntry>, ScanError> {
    if !root.exists() {
        return Err(ScanError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", root.display()),
        )));
    }
    directories::scan_tree_filtered(root, categories)
}

/// Map file extension to a default category.
pub fn categorize_by_extension(ext: &str) -> Category {
    match ext.to_lowercase().as_str() {
        "aex" | "dll" | "so" | "dylib" => Category::Plugins,
        "jsx" | "js" | "jsxbin" => Category::Scripts,
        "zxp" => Category::CEPExtensions,
        "ffx" | "prfpset" => Category::Presets,
        "xml" => Category::Workspaces,
        // Don't categorize .txt as KeyboardShortcuts - let path-based categorization handle it
        _ => Category::Unknown,
    }
}

/// Categorize a file by its path structure within AE directories.
/// Falls back to extension-based categorization.
pub fn categorize_by_path(file_path: &std::path::Path, scan_root: &std::path::Path, extension: &str) -> Category {
    directories::categorize_by_path(file_path, scan_root, extension)
}

/// Compare two FileEntry objects.
/// Returns Ok(true) if they are considered identical (same sha256 and relative path).
/// Returns Ok(false) if they differ in content (sha256) or relative path.
pub fn compare_files(a: &FileEntry, b: &FileEntry) -> Result<bool, ScanError> {
    if a.relative_path != b.relative_path {
        return Ok(false);
    }
    if a.sha256 != b.sha256 {
        return Ok(false);
    }
    Ok(true)
}
