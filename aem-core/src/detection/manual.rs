use crate::detection::parse_version;
use crate::models::{DetectionMethod, Installation};
use std::path::{Path, PathBuf};

/// Validate that a manually supplied path looks like a real AE installation
/// and create an Installation record for it.
pub fn validate_and_create(path: PathBuf) -> Result<Installation, crate::detection::DetectionError> {
    if !path.exists() {
        return Err(crate::detection::DetectionError::InvalidManualPath(format!(
            "path does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(crate::detection::DetectionError::InvalidManualPath(format!(
            "path is not a directory: {}",
            path.display()
        )));
    }
    if !is_complete_ae_install(&path) {
        return Err(crate::detection::DetectionError::InvalidManualPath(format!(
            "directory does not contain After Effects Support Files: {}",
            path.display()
        )));
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("After Effects")
        .to_string();

    Ok(Installation {
        id: format!("manual-{}", path.display()),
        version: parse_version(&extract_version(&name)),
        install_path: path.to_string_lossy().to_string(),
        support_files_path: path.join("Support Files").to_string_lossy().to_string(),
        detection_method: DetectionMethod::Manual,
        is_complete: true,
    })
}

fn is_complete_ae_install(root: &Path) -> bool {
    root.join("Support Files").join("Plug-ins").exists()
        || root.join("Support Files").join("Scripts").exists()
}

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
    fn validate_and_create_rejects_nonexistent_path() {
        let result = validate_and_create(PathBuf::from(r"C:\Nonexistent\Path"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_and_create_rejects_incomplete_directory() {
        let temp = tempfile::tempdir().unwrap();
        let result = validate_and_create(temp.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn validate_and_create_accepts_complete_directory() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Support Files").join("Plug-ins")).unwrap();
        let result = validate_and_create(temp.path().to_path_buf());
        assert!(result.is_ok());
        let inst = result.unwrap();
        assert!(inst.is_complete);
    }

    #[test]
    fn extract_version_pulls_digits() {
        assert_eq!(extract_version("Adobe After Effects 2025"), "2025");
        assert_eq!(extract_version("After Effects 2026"), "2026");
        assert_eq!(extract_version("NoVersion"), "0.0");
    }
}
