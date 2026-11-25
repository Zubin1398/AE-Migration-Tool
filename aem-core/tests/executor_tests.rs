#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

use aem_core::models::{Category, Classification, FileEntry, MigrationPlan, MigrationStep, MigrationStatus, Recommendation, Version};
use std::fs;
use tempfile::tempdir;

fn make_file(relative: &str, sha: &str, content: &str) -> FileEntry {
    FileEntry {
        path: format!(r"C:\src\{relative}"),
        relative_path: relative.to_string(),
        category: Category::Scripts,
        size: content.len() as u64,
        sha256: sha.to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    }
}

#[test]
fn execute_copies_source_only_files() {
    let temp = tempdir().unwrap();
    let target_dir = temp.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();

    let source = make_file("scripts/new.jsx", "abc123", "alert('new');");
    let plan = MigrationPlan {
        id: "test-plan".to_string(),
        source_version: Version { major: 25, minor: 4, patch: 0, build: None, display_name: "25.4".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26.0".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 0,
        steps: vec![MigrationStep {
            id: "step-1".to_string(),
            source_path: source.path.clone(),
            target_path: target_dir.join("scripts/new.jsx").to_string_lossy().to_string(),
            relative_path: "scripts/new.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::KeepSource,
            status: MigrationStatus::Pending,
            reason: "File exists only in source".to_string(),
        }],
    };

    // Create actual source file
    let source_path = temp.path().join("source");
    fs::create_dir_all(source_path.join("scripts")).unwrap();
    fs::write(source_path.join("scripts/new.jsx"), "alert('new');").unwrap();

    let result = aem_core::migration::execute(&plan, &source_path, &target_dir, &temp.path().join("backups")).expect("execute failed");

    assert_eq!(result.completed_steps, 1);
    assert!(target_dir.join("scripts/new.jsx").exists());
    assert_eq!(fs::read_to_string(target_dir.join("scripts/new.jsx")).unwrap(), "alert('new');");
}

#[test]
fn execute_skips_identical_files() {
    let temp = tempdir().unwrap();
    let target_dir = temp.path().join("target");
    fs::create_dir_all(target_dir.join("scripts")).unwrap();
    fs::write(target_dir.join("scripts/existing.jsx"), "alert('existing');").unwrap();

    let plan = MigrationPlan {
        id: "test-plan".to_string(),
        source_version: Version { major: 25, minor: 4, patch: 0, build: None, display_name: "25.4".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26.0".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 0,
        steps: vec![MigrationStep {
            id: "step-1".to_string(),
            source_path: r"C:\src\scripts\existing.jsx".to_string(),
            target_path: target_dir.join("scripts/existing.jsx").to_string_lossy().to_string(),
            relative_path: "scripts/existing.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::KeepTarget,
            status: MigrationStatus::Pending,
            reason: "Files are identical (same SHA-256)".to_string(),
        }],
    };

    let result = aem_core::migration::execute(&plan, &temp.path(), &target_dir, &temp.path().join("backups")).expect("execute failed");

    assert_eq!(result.completed_steps, 0);
    assert_eq!(result.skipped_steps, 1);
    // File should remain unchanged
    assert_eq!(fs::read_to_string(target_dir.join("scripts/existing.jsx")).unwrap(), "alert('existing');");
}

#[test]
fn execute_creates_backup_before_overwrite() {
    let temp = tempdir().unwrap();
    let target_dir = temp.path().join("target");
    let backup_root = temp.path().join("backups");
    fs::create_dir_all(target_dir.join("scripts")).unwrap();
    fs::write(target_dir.join("scripts/conflict.jsx"), "original").unwrap();

    let plan = MigrationPlan {
        id: "test-plan".to_string(),
        source_version: Version { major: 25, minor: 4, patch: 0, build: None, display_name: "25.4".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26.0".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 1,
        steps: vec![MigrationStep {
            id: "step-1".to_string(),
            source_path: r"C:\src\scripts\conflict.jsx".to_string(),
            target_path: target_dir.join("scripts/conflict.jsx").to_string_lossy().to_string(),
            relative_path: "scripts/conflict.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::BackupAndReplace,
            status: MigrationStatus::Pending,
            reason: "Files differ (SHA-256 mismatch)".to_string(),
        }],
    };

    // Create actual source file with different content
    let source_path = temp.path().join("source");
    fs::create_dir_all(source_path.join("scripts")).unwrap();
    fs::write(source_path.join("scripts/conflict.jsx"), "updated").unwrap();

    let result = aem_core::migration::execute(&plan, &source_path, &target_dir, &backup_root).expect("execute failed");

    assert_eq!(result.completed_steps, 1);
    // Target should be updated
    assert_eq!(fs::read_to_string(target_dir.join("scripts/conflict.jsx")).unwrap(), "updated");
    // Backup should exist
    let backups = aem_core::backup::list_backups(&backup_root).expect("list backups failed");
    assert_eq!(backups.len(), 1);
    // Verify backup contains original content
    let backup_content = fs::read_to_string(&backups[0].backup_path).unwrap();
    assert_eq!(backup_content, "original");
}

#[test]
fn rollback_restores_backed_up_files() {
    let temp = tempdir().unwrap();
    let target_dir = temp.path().join("target");
    let backup_root = temp.path().join("backups");
    fs::create_dir_all(target_dir.join("scripts")).unwrap();
    fs::write(target_dir.join("scripts/conflict.jsx"), "original").unwrap();

    let plan = MigrationPlan {
        id: "test-plan".to_string(),
        source_version: Version { major: 25, minor: 4, patch: 0, build: None, display_name: "25.4".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26.0".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 1,
        steps: vec![MigrationStep {
            id: "step-1".to_string(),
            source_path: r"C:\src\scripts\conflict.jsx".to_string(),
            target_path: target_dir.join("scripts/conflict.jsx").to_string_lossy().to_string(),
            relative_path: "scripts/conflict.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::BackupAndReplace,
            status: MigrationStatus::Pending,
            reason: "Files differ (SHA-256 mismatch)".to_string(),
        }],
    };

    let source_path = temp.path().join("source");
    fs::create_dir_all(source_path.join("scripts")).unwrap();
    fs::write(source_path.join("scripts/conflict.jsx"), "updated").unwrap();

    // Execute migration
    aem_core::migration::execute(&plan, &source_path, &target_dir, &backup_root).expect("execute failed");
    assert_eq!(fs::read_to_string(target_dir.join("scripts/conflict.jsx")).unwrap(), "updated");

    // Rollback
    aem_core::migration::rollback("test-plan", &backup_root).expect("rollback failed");

    // Target should be restored to original
    assert_eq!(fs::read_to_string(target_dir.join("scripts/conflict.jsx")).unwrap(), "original");
    let backups = aem_core::backup::list_backups(&backup_root).unwrap();
    assert!(backups.iter().all(|record| record.rolled_back));
    assert!(aem_core::migration::rollback("test-plan", &backup_root).is_err());
}

#[test]
fn rollback_removes_files_added_by_migration() {
    let temp = tempdir().unwrap();
    let source_root = temp.path().join("source");
    let target_root = temp.path().join("target");
    let backup_root = temp.path().join("backups");
    fs::create_dir_all(source_root.join("scripts")).unwrap();
    fs::write(source_root.join("scripts/new.jsx"), "new file").unwrap();

    let source = make_file("scripts/new.jsx", "new-hash", "new file");
    let plan = MigrationPlan {
        id: "new-file-plan".to_string(),
        source_version: Version { major: 25, minor: 0, patch: 0, build: None, display_name: "25".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 0,
        steps: vec![MigrationStep {
            id: "new-file-step".to_string(),
            source_path: source.path,
            target_path: String::new(),
            relative_path: "scripts/new.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::KeepSource,
            status: MigrationStatus::Pending,
            reason: "File exists only in source".to_string(),
        }],
    };

    aem_core::migration::execute(&plan, &source_root, &target_root, &backup_root).unwrap();
    assert!(target_root.join("scripts/new.jsx").exists());
    aem_core::migration::rollback("new-file-plan", &backup_root).unwrap();
    assert!(!target_root.join("scripts/new.jsx").exists());
}

#[test]
fn keep_target_can_be_backed_up_and_rolled_back() {
    let temp = tempdir().unwrap();
    let target_root = temp.path().join("target");
    let backup_root = temp.path().join("backups");
    let target_file = target_root.join("scripts/conflict.jsx");
    fs::create_dir_all(target_file.parent().unwrap()).unwrap();
    fs::write(&target_file, "target version").unwrap();

    let plan = MigrationPlan {
        id: "keep-target-plan".to_string(),
        source_version: Version { major: 25, minor: 0, patch: 0, build: None, display_name: "25".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 1,
        steps: vec![MigrationStep {
            id: "keep-target-step".to_string(),
            source_path: r"C:\source\scripts\conflict.jsx".to_string(),
            target_path: target_file.to_string_lossy().to_string(),
            relative_path: "scripts/conflict.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::BackupAndKeepTarget,
            status: MigrationStatus::Pending,
            reason: "Files differ".to_string(),
        }],
    };

    let result = aem_core::migration::execute(&plan, temp.path(), &target_root, &backup_root).unwrap();
    assert_eq!(result.skipped_steps, 1);
    assert_eq!(fs::read_to_string(&target_file).unwrap(), "target version");
    fs::write(&target_file, "changed later").unwrap();

    aem_core::migration::rollback("keep-target-plan", &backup_root).unwrap();
    assert_eq!(fs::read_to_string(&target_file).unwrap(), "target version");
}

#[test]
fn rollback_restores_same_named_files_from_distinct_folders() {
    let temp = tempdir().unwrap();
    let backup_root = temp.path().join("backups");
    let first = temp.path().join("target/one/settings.jsx");
    let second = temp.path().join("target/two/settings.jsx");
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    fs::create_dir_all(second.parent().unwrap()).unwrap();
    fs::write(&first, "first original").unwrap();
    fs::write(&second, "second original").unwrap();

    aem_core::backup::create_backup(&first, &backup_root, "duplicate-name-plan").unwrap();
    aem_core::backup::create_backup(&second, &backup_root, "duplicate-name-plan").unwrap();
    fs::write(&first, "first changed").unwrap();
    fs::write(&second, "second changed").unwrap();

    aem_core::migration::rollback("duplicate-name-plan", &backup_root).unwrap();
    assert_eq!(fs::read_to_string(&first).unwrap(), "first original");
    assert_eq!(fs::read_to_string(&second).unwrap(), "second original");
}

#[test]
fn rollback_without_backup_data_returns_an_error() {
    let temp = tempdir().unwrap();
    let result = aem_core::migration::rollback("missing-plan", &temp.path().join("backups"));
    assert!(result.is_err());
}
