use crate::detection::parse_version;
use crate::models::{DetectionMethod, Installation};
use std::path::PathBuf;
use thiserror::Error;

/// Known registry locations where AE installs may register themselves.
const UNINSTALL_KEYS: &[&str] = &[
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
];

/// Detect AE installations via Windows registry uninstall entries.
pub fn detect_registry() -> Result<Vec<Installation>, crate::detection::DetectionError> {
    let mut installations = Vec::new();

    for key_path in UNINSTALL_KEYS {
        let hk_lm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        let uninstall_key = match hk_lm.open_subkey(key_path) {
            Ok(k) => k,
            Err(_) => continue,
        };

        for sub in uninstall_key.enum_keys().flatten() {
            if let Ok(sub_key) = uninstall_key.open_subkey(&sub) {
                if let Some(inst) = try_read_ae_install(&sub, &sub_key) {
                    installations.push(inst);
                }
            }
        }
    }

    Ok(installations)
}

/// Try to read an AE installation from a single uninstall subkey.
fn try_read_ae_install(sub_name: &str, key: &winreg::RegKey) -> Option<Installation> {
    let display_name: String = key.get_value("DisplayName").ok()?;
    if !display_name.to_lowercase().contains("after effects") {
        return None;
    }

    let install_location: Option<String> = key.get_value("InstallLocation").ok();
    let version_string: Option<String> = key.get_value("DisplayVersion").ok();

    let install_path = install_location?;
    let path = PathBuf::from(&install_path);
    if !path.join("Support Files").exists() {
        return None;
    }

    let version = version_string
        .as_deref()
        .map(parse_version)
        .unwrap_or_else(|| parse_version(&extract_version(&display_name)));

    Some(Installation {
        id: format!("reg-{}", sub_name),
        version,
        install_path,
        support_files_path: path.join("Support Files").to_string_lossy().to_string(),
        detection_method: DetectionMethod::Registry,
        is_complete: true,
    })
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

    #[test]
    fn extract_version_pulls_digits() {
        assert_eq!(extract_version("Adobe After Effects 2025"), "2025");
        assert_eq!(extract_version("After Effects 2026 (64-bit)"), "202664");
        assert_eq!(extract_version("NoVersion"), "0.0");
    }
}
