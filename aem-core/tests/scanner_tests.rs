#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

use aem_core::models::{Category, Classification, FileEntry};

#[test]
fn scan_directory_collects_files_with_metadata() {
    use std::fs;
    use std::path::PathBuf;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    fs::create_dir_all(root.join("sub").join("deep")).unwrap();
    fs::write(root.join("a.jsx"), "alert('a');").unwrap();
    fs::write(root.join("sub").join("b.jsx"), "alert('b');").unwrap();
    fs::write(root.join("sub").join("deep").join("c.jsx"), "alert('c');").unwrap();

    let files = aem_core::scanner::scan_directory(&root).expect("scan failed");

    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    assert!(paths.iter().any(|p| p.ends_with("a.jsx")));
    assert!(paths.iter().any(|p| p.ends_with("b.jsx")));
    assert!(paths.iter().any(|p| p.ends_with("c.jsx")));

    // every file should have a non-empty sha256 and a size
    for f in &files {
        assert!(!f.sha256.is_empty(), "sha256 should not be empty for {:?}", f.path);
        assert!(f.size > 0, "size should be > 0 for {:?}", f.path);
        assert!(f.extension == "jsx", "expected jsx extension, got {:?}", f.extension);
    }
}

#[test]
fn scan_directory_skips_nonexistent_root() {
    use std::path::PathBuf;
    let result = aem_core::scanner::scan_directory(&PathBuf::from(r"C:\Nonexistent\Path"));
    assert!(result.is_err());
}

#[test]
fn scan_directory_ignores_binary_placeholder() {
    use std::fs;
    use std::path::PathBuf;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    fs::write(root.join("data.bin"), &[0u8; 100]).unwrap();

    let files = aem_core::scanner::scan_directory(&root).expect("scan failed");
    let bin_files: Vec<_> = files.iter().filter(|f| f.extension == "bin").collect();
    assert!(bin_files.is_empty(), "binary files should be skipped by default");
}

#[test]
fn compare_files_matches_same_sha256() {
    let sha = "abc123".to_string();
    let file1 = FileEntry {
        path: r"C:\path\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: sha.clone(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    let file2 = FileEntry {
        path: r"C:\other\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: sha,
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    assert_eq!(aem_core::scanner::compare_files(&file1, &file2).unwrap(), true);
}

#[test]
fn compare_files_differs_different_sha256() {
    let file1 = FileEntry {
        path: r"C:\path\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    let file2 = FileEntry {
        path: r"C:\other\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "xyz789".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    assert_eq!(aem_core::scanner::compare_files(&file1, &file2).unwrap(), false);
}

#[test]
fn compare_files_differs_different_path() {
    let file1 = FileEntry {
        path: r"C:\path\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    let file2 = FileEntry {
        path: r"D:\other\a.jsx".to_string(),
        relative_path: "a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    // Files with same relative path and SHA-256 are considered identical, even if absolute paths differ.
    assert_eq!(aem_core::scanner::compare_files(&file1, &file2).unwrap(), true);
}
