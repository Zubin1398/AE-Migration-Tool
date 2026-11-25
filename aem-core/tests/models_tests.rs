use aem_core::models::*;

#[test]
fn installation_round_trip() {
    let inst = Installation {
        id: "test-1".to_string(),
        version: Version {
            major: 25,
            minor: 4,
            patch: 0,
            build: None,
            display_name: "Adobe After Effects 2025".to_string(),
        },
        install_path: r"C:\Program Files\Adobe\Adobe After Effects 2025".to_string(),
        support_files_path: r"C:\Program Files\Adobe\Adobe After Effects 2025\Support Files".to_string(),
        detection_method: DetectionMethod::Filesystem,
        is_complete: true,
    };
    let json = serde_json::to_string(&inst).unwrap();
    let back: Installation = serde_json::from_str(&json).unwrap();
    assert_eq!(inst, back);
    assert_eq!(inst.version.display_name, "Adobe After Effects 2025");
}

#[test]
fn file_entry_has_sha256_and_size() {
    let entry = FileEntry {
        path: r"C:\source\Support Files\Plug-ins\Custom.aex".to_string(),
        relative_path: r"Support Files\Plug-ins\Custom.aex".to_string(),
        category: Category::Plugins,
        size: 12345,
        sha256: "abc123".to_string(),
        extension: "aex".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    assert_eq!(entry.size, 12345);
    assert_eq!(entry.category, Category::Plugins);
}

#[test]
fn classification_defaults_to_unknown_low() {
    let c = Classification::default();
    assert_eq!(c.category, Category::Unknown);
    assert_eq!(c.confidence, Confidence::Low);
}

#[test]
fn migration_status_enum_serializes() {
    let s = MigrationStatus::Completed;
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(json, "\"completed\"");
    let back: MigrationStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}
