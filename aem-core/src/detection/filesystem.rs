use crate::detection::parse_version;
use crate::models::{DetectionMethod, Installation, Version};
use std::path::{Path, PathBuf};

/// Standard root directories where Adobe installs After Effects.
pub(crate) fn adobe_install_roots() -> Vec<PathBuf> {
    vec![
        PathBuf::from(r"C:\Program Files\Adobe"),
        PathBuf::from(r"C:\Program Files (x86)\Adobe"),
    ]
}

/// Check whether a directory looks like a complete AE installation.
fn is_complete_ae_install(root: &Path) -> bool {
    root.join("Support Files").join("Plug-ins").exists()
        || root.join("Support Files").join("Scripts").exists()
}

/// Detect AE installations by scanning the given root directories.
pub fn detect_filesystem(roots: &[PathBuf]) -> Result<Vec<Installation>, crate::detection::DetectionError> {
    let mut installations = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };
                // Match directories like "Adobe After Effects 2025" or "After Effects 2026"
                if !name.to_lowercase().contains("after effects") {
                    continue;
                }
                if !is_complete_ae_install(&path) {
                    continue;
                }
                let version = parse_version(&extract_version(&name));
                installations.push(Installation {
                    id: format!("fs-{}", path.display()),
                    version,
                    install_path: path.to_string_lossy().to_string(),
                    support_files_path: path.join("Support Files").to_string_lossy().to_string(),
                    detection_method: DetectionMethod::Filesystem,
                    is_complete: true,
                });
            }
        }
    }

    Ok(installations)
}

/// Extract a version number from a directory name like "Adobe After Effects 2025".
fn extract_version(name: &str) -> String {
    let digits: String = name.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if digits.is_empty() {
        "0.0".to_string()
    } else {
        digits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn is_complete_ae_install_detects_support_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("Support Files").join("Plug-ins")).unwrap();
        assert!(is_complete_ae_install(root));
    }

    #[test]
    fn is_complete_ae_install_rejects_missing_support_files() {
        let temp = tempfile::tempdir().unwrap();
        assert!(!is_complete_ae_install(temp.path()));
    }

    #[test]
    fn extract_version_pulls_digits() {
        assert_eq!(extract_version("Adobe After Effects 2025"), "2025");
        assert_eq!(extract_version("After Effects 2026"), "2026");
        assert_eq!(extract_version("NoVersion"), "0.0");
    }
}
