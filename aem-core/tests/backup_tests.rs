#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

use aem_core::models::BackupRecord;
use std::fs;
use tempfile::tempdir;

#[test]
fn backup_creates_copy_with_metadata() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("source.txt");
    let backup_root = temp.path().join("backups");
    fs::write(&source, "hello world").unwrap();

    let backup_id = aem_core::backup::create_backup(&source, &backup_root, "migration-123")
        .expect("backup failed");

    // Verify backup file exists
    let backup_path = backup_root.join("migration-123").join("source.txt");
    assert!(backup_path.exists(), "backup file should exist");

    // Verify content matches
    let content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(content, "hello world");

    // Verify backup record
    let records = aem_core::backup::list_backups(&backup_root).expect("list failed");
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.migration_id, "migration-123");
    assert_eq!(record.original_path, source.to_string_lossy().to_string());
    assert!(record.backup_path.ends_with("source.txt"));
}

#[test]
fn backup_preserves_file_name() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("scripts").join("a.jsx");
    let backup_root = temp.path().join("backups");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "alert('a');").unwrap();

    let _ = aem_core::backup::create_backup(&source, &backup_root, "migration-123")
        .expect("backup failed");

    let backup_path = backup_root.join("migration-123").join("a.jsx");
    assert!(backup_path.exists(), "backup should preserve file name");
    assert_eq!(
        fs::read_to_string(&backup_path).unwrap(),
        "alert('a');"
    );
}

#[test]
fn backup_verify_integrity() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("source.txt");
    let backup_root = temp.path().join("backups");
    fs::write(&source, "hello world").unwrap();

    let _ = aem_core::backup::create_backup(&source, &backup_root, "migration-123")
        .expect("backup failed");

    let records = aem_core::backup::list_backups(&backup_root).expect("list failed");
    let record = &records[0];

    let ok = aem_core::backup::verify_integrity(record).expect("verify failed");
    assert!(ok, "backup integrity should be valid");
}

#[test]
fn backup_fails_for_nonexistent_source() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("nonexistent.txt");
    let backup_root = temp.path().join("backups");

    let result = aem_core::backup::create_backup(&source, &backup_root, "migration-123");
    assert!(result.is_err(), "backup should fail for nonexistent source");
}