#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

use aem_core::{
    detection::{add_manual, detect_installations},
    migration::{dry_run, execute, rollback},
    scanner::scan_directory,
    backup::list_backups,
    models::{Installation, Version, Category, Classification, FileEntry, MigrationPlan, MigrationStep, MigrationStatus, Recommendation},
};
use std::fs;
use tempfile::tempdir;

fn make_file_entry(relative: &str, sha: &str, content: &str) -> FileEntry {
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

fn create_fake_ae_install(root: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("Support Files").join("Plug-ins"))?;
    fs::create_dir_all(root.join("Support Files").join("Scripts"))?;
    fs::write(root.join("Support Files").join("AfterFX.exe"), "fake")?;
    Ok(())
}

#[test]
fn full_migration_cycle() {
    let temp = tempdir().unwrap();

    // Setup source (AE 25.4) and target (AE 26.0) directories
    let source_root = temp.path().join("ae-25.4");
    let target_root = temp.path().join("ae-26.0");
    let backup_root = temp.path().join("backups");

    fs::create_dir_all(source_root.join("Scripts")).unwrap();
    fs::create_dir_all(target_root.join("Scripts")).unwrap();

    // Source has new file and updated file
    fs::write(source_root.join("Scripts/new_script.jsx"), "alert('new');").unwrap();
    fs::write(source_root.join("Scripts/existing.jsx"), "alert('updated');").unwrap();

    // Target has existing file (different content) and target-only file
    fs::write(target_root.join("Scripts/existing.jsx"), "alert('original');").unwrap();
    fs::write(target_root.join("Scripts/target_only.jsx"), "alert('target only');").unwrap();

    // 1. Detect installations (manual)
    let source_root = temp.path().join("ae-25.4");
    let target_root = temp.path().join("ae-26.0");
    create_fake_ae_install(&source_root).unwrap();
    create_fake_ae_install(&target_root).unwrap();

    let source_inst = add_manual(source_root.clone()).expect("add source failed");
    let target_inst = add_manual(target_root.clone()).expect("add target failed");
    assert_eq!(source_inst.version.major, 25);
    assert_eq!(target_inst.version.major, 26);

    // 2. Scan both directories
    let source_files = scan_directory(&source_root).expect("scan source failed");
    let target_files = scan_directory(&target_root).expect("scan target failed");

    assert_eq!(source_files.len(), 2);
    assert_eq!(target_files.len(), 2);

    // 3. Generate dry-run plan
    let plan = dry_run(&source_files, &target_files).expect("dry run failed");
    assert_eq!(plan.steps.len(), 3); // new_script, existing, target_only

    // Verify step recommendations
    let new_script_step = plan.steps.iter().find(|s| s.relative_path.ends_with("new_script.jsx")).unwrap();
    assert_eq!(new_script_step.recommendation, Recommendation::KeepSource);

    let existing_step = plan.steps.iter().find(|s| s.relative_path.ends_with("existing.jsx")).unwrap();
    assert_eq!(existing_step.recommendation, Recommendation::KeepTarget); // default for conflict

    let target_only_step = plan.steps.iter().find(|s| s.relative_path.ends_with("target_only.jsx")).unwrap();
    assert_eq!(target_only_step.recommendation, Recommendation::Skip);

    // 4. Execute migration (with backup for conflicts)
    let result = execute(&plan, &source_root, &target_root, &backup_root).expect("execute failed");
    assert_eq!(result.completed_steps, 1); // only new_script copied
    assert_eq!(result.skipped_steps, 2); // existing (KeepTarget) + target_only (Skip)

    // Verify target state after migration
    assert!(target_root.join("Scripts/new_script.jsx").exists());
    assert_eq!(fs::read_to_string(target_root.join("Scripts/new_script.jsx")).unwrap(), "alert('new');");
    assert_eq!(fs::read_to_string(target_root.join("Scripts/existing.jsx")).unwrap(), "alert('original');"); // unchanged
    assert_eq!(fs::read_to_string(target_root.join("Scripts/target_only.jsx")).unwrap(), "alert('target only');"); // unchanged

    // The added file is tracked so rollback can remove it later.
    let backups = list_backups(&backup_root).expect("list backups failed");
    assert_eq!(backups.len(), 1);
    assert!(backups[0].created_by_migration);

    // existing.jsx should remain unchanged (KeepTarget)
    assert_eq!(fs::read_to_string(target_root.join("Scripts/existing.jsx")).unwrap(), "alert('original');");
}

#[test]
fn migration_with_backup_and_replace() {
    let temp = tempdir().unwrap();

    let source_root = temp.path().join("ae-25.4");
    let target_root = temp.path().join("ae-26.0");
    let backup_root = temp.path().join("backups");

    fs::create_dir_all(source_root.join("Scripts")).unwrap();
    fs::create_dir_all(target_root.join("Scripts")).unwrap();

    // Same file, different content
    fs::write(source_root.join("Scripts/script.jsx"), "alert('source');").unwrap();
    fs::write(target_root.join("Scripts/script.jsx"), "alert('target');").unwrap();

    let source_files = scan_directory(&source_root).unwrap();
    let target_files = scan_directory(&target_root).unwrap();

    // Manually create plan with BackupAndReplace recommendation
    let plan = MigrationPlan {
        id: "test-backup-replace".to_string(),
        source_version: Version { major: 25, minor: 4, patch: 0, build: None, display_name: "25.4".to_string() },
        target_version: Version { major: 26, minor: 0, patch: 0, build: None, display_name: "26.0".to_string() },
        dry_run: false,
        total_files: 1,
        conflicts: 1,
        steps: vec![MigrationStep {
            id: "step-1".to_string(),
            source_path: source_root.join("Scripts/script.jsx").to_string_lossy().to_string(),
            target_path: target_root.join("Scripts/script.jsx").to_string_lossy().to_string(),
            relative_path: "Scripts/script.jsx".to_string(),
            category: Category::Scripts,
            recommendation: Recommendation::BackupAndReplace,
            status: MigrationStatus::Pending,
            reason: "Force replace".to_string(),
        }],
    };

    let result = execute(&plan, &source_root, &target_root, &backup_root).unwrap();
    assert_eq!(result.completed_steps, 1);

    // Target should now have source content
    assert_eq!(fs::read_to_string(target_root.join("Scripts/script.jsx")).unwrap(), "alert('source');");

    // Backup should have target content
    let backups = list_backups(&backup_root).unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read_to_string(&backups[0].backup_path).unwrap(), "alert('target');");

    // Rollback restores target content
    rollback("test-backup-replace", &backup_root).unwrap();
    assert_eq!(fs::read_to_string(target_root.join("Scripts/script.jsx")).unwrap(), "alert('target');");
}

#[test]
fn detect_installations_integration() {
    let temp = tempdir().unwrap();

    // Create fake AE installations with proper structure
    let ae25 = temp.path().join("Adobe After Effects 2025");
    let ae26 = temp.path().join("Adobe After Effects 2026");

    create_fake_ae_install(&ae25).unwrap();
    create_fake_ae_install(&ae26).unwrap();

    // Add manually
    let inst25 = add_manual(ae25).unwrap();
    let inst26 = add_manual(ae26).unwrap();

    assert_eq!(inst25.version.major, 25);
    assert_eq!(inst26.version.major, 26);

    // Detect should find both
    let installations = detect_installations().unwrap();
    assert!(installations.len() >= 2);
}
