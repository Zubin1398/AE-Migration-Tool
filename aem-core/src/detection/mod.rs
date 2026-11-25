pub mod filesystem;
pub mod registry;
pub mod manual;

use crate::models::{DetectionMethod, Installation, Version};
use thiserror::Error;

/// Error during installation detection.
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("registry access failed: {0}")]
    Registry(String),
    #[error("filesystem access failed: {0}")]
    Filesystem(String),
    #[error("invalid manual path: {0}")]
    InvalidManualPath(String),
}

/// Detect all installed After Effects versions.
///
/// Combines filesystem checks, registry checks, and any manually added paths.
pub fn detect_installations() -> Result<Vec<Installation>, DetectionError> {
    let mut installations = Vec::new();

    // Filesystem-based detection
    if let Ok(fs_installs) = filesystem::detect_filesystem(&filesystem::adobe_install_roots()) {
        installations.extend(fs_installs);
    }

    // Registry-based detection
    if let Ok(reg_installs) = registry::detect_registry() {
        installations.extend(reg_installs);
    }

    // Deduplicate by install path
    installations.dedup_by(|a, b| a.install_path == b.install_path);

    Ok(installations)
}

/// Manually add an installation path. Validates it looks like a real AE install.
pub fn add_manual(path: std::path::PathBuf) -> Result<Installation, DetectionError> {
    manual::validate_and_create(path)
}

/// Parse a version string like "25.4" or "25.4.0" into a Version.
/// Also handles year-based directory names like "2025" -> major 25.
pub fn parse_version(display: &str) -> Version {
    let parts: Vec<&str> = display.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

    // If major looks like a year (2020-2030), convert to AE major version
    let major = if (2020..=2030).contains(&major) {
        major - 2000
    } else {
        major
    };

    Version {
        major,
        minor,
        patch,
        build: None,
        display_name: display.to_string(),
    }
}
