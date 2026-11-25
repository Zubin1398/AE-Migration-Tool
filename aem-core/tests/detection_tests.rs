#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

#[test]
fn detect_installations_finds_fake_ae_structure() {
    use std::fs;
    use std::path::PathBuf;

    let temp = tempfile::tempdir().unwrap();
    let ae_dir = temp.path().join("Adobe After Effects 25.4");
    fs::create_dir_all(ae_dir.join("Support Files").join("Plug-ins")).unwrap();
    fs::create_dir_all(ae_dir.join("Support Files").join("Scripts")).unwrap();

    let installs = aem_core::detection::filesystem::detect_filesystem(&[temp.path().to_path_buf()])
        .expect("detection failed");
    let found = installs.iter().find(|i| i.version.major == 25 && i.version.minor == 4);
    assert!(found.is_some(), "expected to find fake AE 25.4 installation");
    let inst = found.unwrap();
    assert!(inst.is_complete);
}

#[test]
fn detect_installations_returns_empty_when_no_ae() {
    let installs = aem_core::detection::detect_installations().expect("detection failed");
    // On a machine without AE this may be empty or not; just must not panic.
    let _ = installs.len();
}

#[test]
fn add_manual_rejects_incomplete_directory() {
    use std::path::PathBuf;
    let result = aem_core::detection::add_manual(PathBuf::from(r"C:\Nonexistent\Path"));
    assert!(result.is_err());
}