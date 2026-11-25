use crate::models::{Category, FileEntry};
use crate::scanner::{ScanError, categorize_by_extension};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use walkdir::WalkDir;

/// Scan a directory tree and return all files with metadata.
pub fn scan_tree(root: &Path) -> Result<Vec<FileEntry>, ScanError> {
    let mut entries = Vec::new();

    // Define user-content directories to scan within AE installation
    let support_files = root.join("Support Files");
    let scan_dirs = if support_files.is_dir() {
        let mut dirs = get_user_content_dirs(root);
        // Some installations and user-managed layouts keep these folders at
        // the installation root instead of under Support Files.
        dirs.extend([
            root.join("Presets"),
            root.join("Scripts"),
            root.join("Plug-ins"),
            root.join("CEP").join("extensions"),
            root.join("Workspaces"),
            root.join("Keyboard Shortcuts"),
            root.join("Preferences"),
        ]);
        dirs
    } else {
        // Keep generic directory scanning useful for manually selected folders
        // and for installations whose layout differs from the usual AE layout.
        vec![root.to_path_buf()]
    };

    for scan_root in scan_dirs {
        if !scan_root.exists() {
            continue;
        }

        for entry in WalkDir::new(&scan_root).follow_links(false) {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Skip known runtime/temporary files
            if should_skip_file(path) {
                continue;
            }

            // Skip binary files by content heuristic
            if is_binary_file(path) {
                continue;
            }

            let metadata = fs::metadata(path)?;
            let size = metadata.len();

            let sha256 = compute_sha256(path)?;

            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Categorize by both extension AND path structure
            let category = categorize_by_path(path, &scan_root, &extension);

            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            entries.push(FileEntry {
                path: path.to_string_lossy().to_string(),
                relative_path,
                category,
                size,
                sha256,
                extension,
                last_modified: metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
                classification: Default::default(),
            });
        }
    }

    // Deduplicate by relative_path (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(e.relative_path.clone()));

    Ok(entries)
}

/// Scan a directory tree and return files filtered by categories.
pub fn scan_tree_filtered(root: &Path, categories: &[Category]) -> Result<Vec<FileEntry>, ScanError> {
    let all_files = scan_tree(root)?;
    let category_set: std::collections::HashSet<_> = categories.iter().collect();
    Ok(all_files.into_iter().filter(|f| category_set.contains(&f.category)).collect())
}

/// Get user-content directories to scan within an AE installation.
/// These are the folders where users store their custom content.
fn get_user_content_dirs(root: &Path) -> Vec<PathBuf> {
    let support_files = root.join("Support Files");
    let mut dirs = Vec::new();

    // User presets folder
    dirs.push(support_files.join("Presets"));
    // User scripts folder
    dirs.push(support_files.join("Scripts"));
    // User ScriptUI Panels
    dirs.push(support_files.join("Scripts").join("ScriptUI Panels"));
    // User plug-ins (third-party)
    dirs.push(support_files.join("Plug-ins"));
    // CEP Extensions
    dirs.push(support_files.join("CEP").join("extensions"));
    // Workspaces
    dirs.push(support_files.join("Workspaces"));
    // Keyboard Shortcuts
    dirs.push(support_files.join("Keyboard Shortcuts"));
    // Preferences (user prefs)
    dirs.push(support_files.join("Preferences"));
    // Media/Templates/Shared Resources
    dirs.push(support_files.join("Media"));
    dirs.push(support_files.join("Templates"));
    dirs.push(support_files.join("Required"));

    dirs
}

/// Categorize a file by its path structure within AE directories.
/// Falls back to extension-based categorization.
pub fn categorize_by_path(file_path: &Path, scan_root: &Path, extension: &str) -> Category {
    // Get relative path from the scan root
    let relative = file_path.strip_prefix(scan_root).unwrap_or(file_path);
    let path_str = relative.to_string_lossy().to_lowercase();
    let parent = file_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("").to_lowercase();

    // ScriptUI Panels lives below Scripts, so detect it before matching the
    // broader Scripts scan root.
    if path_str.contains("scriptui panels") || path_str.contains("scriptui-panels") || parent.contains("scriptui") {
        return Category::ScriptUIPanels;
    }

    // Also check the scan_root path itself for category hints
    let scan_root_str = scan_root.to_string_lossy().to_lowercase();
    let sep = MAIN_SEPARATOR.to_string().to_lowercase();

    // Categorize by scan_root directory (most reliable for AE structure)
    if scan_root_str.contains(&format!("{}presets", sep)) || scan_root_str.contains("/presets") {
        return Category::Presets;
    }
    if scan_root_str.contains(&format!("{}scripts{}scriptui panels", sep, sep)) || scan_root_str.contains("/scripts/scriptui panels") {
        return Category::ScriptUIPanels;
    }
    if scan_root_str.contains(&format!("{}scripts", sep)) || scan_root_str.contains("/scripts") {
        return Category::Scripts;
    }
    if scan_root_str.contains(&format!("{}plug-ins", sep)) || scan_root_str.contains("/plug-ins") || scan_root_str.contains(&format!("{}plugins", sep)) || scan_root_str.contains("/plugins") {
        return Category::Plugins;
    }
    if scan_root_str.contains(&format!("{}cep{}extensions", sep, sep)) || scan_root_str.contains("/cep/extensions") {
        return Category::CEPExtensions;
    }
    if scan_root_str.contains(&format!("{}workspaces", sep)) || scan_root_str.contains("/workspaces") {
        return Category::Workspaces;
    }
    if scan_root_str.contains(&format!("{}keyboard shortcuts", sep)) || scan_root_str.contains("/keyboard shortcuts") {
        return Category::KeyboardShortcuts;
    }
    if scan_root_str.contains(&format!("{}preferences", sep)) || scan_root_str.contains("/preferences") {
        return Category::Preferences;
    }
    if scan_root_str.contains(&format!("{}media", sep)) || scan_root_str.contains("/media") ||
       scan_root_str.contains(&format!("{}templates", sep)) || scan_root_str.contains("/templates") ||
       scan_root_str.contains(&format!("{}required", sep)) || scan_root_str.contains("/required") {
        return Category::SharedResource;
    }

    // Categorize by parent folder name / path structure (fallback for nested structures)
    if path_str.contains("preset") || parent.contains("preset") {
        return Category::Presets;
    }
    if path_str.contains("script") {
        if path_str.contains("scriptui") || path_str.contains("script ui") || parent.contains("scriptui") {
            return Category::ScriptUIPanels;
        }
        return Category::Scripts;
    }
    if path_str.contains("plug-in") || path_str.contains("plugin") {
        return Category::Plugins;
    }
    if path_str.contains("cep") || path_str.contains("extension") {
        return Category::CEPExtensions;
    }
    if path_str.contains("workspace") {
        return Category::Workspaces;
    }
    if path_str.contains("keyboard") || path_str.contains("shortcut") {
        return Category::KeyboardShortcuts;
    }
    if path_str.contains("preference") || path_str.contains("prefs") {
        return Category::Preferences;
    }
    if path_str.contains("media") || path_str.contains("template") || path_str.contains("required") {
        return Category::SharedResource;
    }

    // Fallback to extension-based categorization
    categorize_by_extension(extension)
}

/// Compute SHA-256 hash of a file (streaming, no full file in memory).
fn compute_sha256(path: &Path) -> Result<String, ScanError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if a file should be skipped (runtime/temporary files).
fn should_skip_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();

    // Skip known runtime/temporary files
    if file_name.contains("plugin loading.log") ||
       file_name.contains("trace database") ||
       file_name.contains("debug database") ||
       file_name.contains("scrpriorstate") ||
       file_name == "lock" ||
       file_name == "current" ||
       file_name.ends_with(".ldb") ||
       file_name.ends_with(".log") {
        return true;
    }

    // Skip UXP PluginStorage directories entirely
    if path.components().any(|c| c.as_os_str() == "PluginStorage") {
        return true;
    }

    false
}

/// Heuristic: a file is considered binary if it contains a null byte
/// within the first 4 KB.
/// But we don't skip known AE file types even if they're binary.
fn is_binary_file(path: &Path) -> bool {
    // Don't skip known AE binary file types
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if matches!(ext.as_str(), "aex" | "ffx" | "prfpset" | "jsxbin" | "dll" | "so" | "dylib" | "zxp") {
        return false;
    }

    match fs::File::open(path) {
        Ok(mut file) => {
            let mut buffer = [0u8; 4096];
            match std::io::Read::read(&mut file, &mut buffer) {
                Ok(bytes_read) => buffer[..bytes_read].contains(&0),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_tree_collects_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("a.jsx"), "alert('a');").unwrap();
        fs::write(root.join("sub").join("b.jsx"), "alert('b');").unwrap();

        let files = scan_tree(root).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn scan_tree_computes_sha256() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::write(root.join("test.txt"), "hello").unwrap();

        let files = scan_tree(root).unwrap();
        assert_eq!(files.len(), 1);
        assert!(!files[0].sha256.is_empty());
        assert_eq!(files[0].sha256.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn scan_tree_skips_plugin_loading_log() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::write(root.join("Plugin Loading.log"), "log data").unwrap();
        fs::write(root.join("real.txt"), "data").unwrap();

        let files = scan_tree(root).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.ends_with("real.txt"), true);
    }
}
