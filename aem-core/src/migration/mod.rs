pub use crate::models::{Category, FileEntry, MigrationPlan, MigrationStep, MigrationStatus, Recommendation, Version, MigrationResult, MigrationHistoryEntry};
use crate::scanner::compare_files;
use crate::backup::create_backup;
use thiserror::Error;
use std::fs;
use std::path::Path;

/// Error during migration plan generation.
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scan error: {0}")]
    Scan(#[from] crate::scanner::ScanError),
    #[error("backup error: {0}")]
    Backup(#[from] crate::backup::BackupError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no rollback data was found for migration {0}")]
    NoRollbackData(String),
    #[error("migration {0} has already been rolled back")]
    AlreadyRolledBack(String),
}

/// Generate a dry-run migration plan from source and target file lists.
pub fn dry_run(
    source_files: &[FileEntry],
    target_files: &[FileEntry],
) -> Result<MigrationPlan, MigrationError> {
    let mut steps = Vec::new();
    let mut conflicts = 0;

    // Build a map of target files by relative path for quick lookup
    let target_map: std::collections::HashMap<_, _> = target_files
        .iter()
        .map(|f| (f.relative_path.clone(), f))
        .collect();

    // Process source files
    for source in source_files {
        if let Some(target) = target_map.get(&source.relative_path) {
            // File exists in both - compare
            let same = compare_files(source, target)?;
            if same {
                // Identical - keep target
                steps.push(MigrationStep {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_path: source.path.clone(),
                    target_path: target.path.clone(),
                    relative_path: source.relative_path.clone(),
                    category: source.category.clone(),
                    recommendation: Recommendation::KeepTarget,
                    status: MigrationStatus::Pending,
                    reason: "Файлы идентичны: SHA-256 совпадает.".to_string(),
                });
            } else {
                // Conflict - default to keep target (safe)
                conflicts += 1;
                steps.push(MigrationStep {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_path: source.path.clone(),
                    target_path: target.path.clone(),
                    relative_path: source.relative_path.clone(),
                    category: source.category.clone(),
                    recommendation: Recommendation::KeepTarget,
                    status: MigrationStatus::Pending,
                    reason: "Файл есть в обеих версиях, но содержимое разное (SHA-256 не совпадает).".to_string(),
                });
            }
        } else {
            // Source only - copy to target
            steps.push(MigrationStep {
                id: uuid::Uuid::new_v4().to_string(),
                source_path: source.path.clone(),
                target_path: source.relative_path.clone(),
                relative_path: source.relative_path.clone(),
                category: source.category.clone(),
                recommendation: Recommendation::KeepSource,
                status: MigrationStatus::Pending,
                reason: "Файл есть только в источнике.".to_string(),
            });
        }
    }

    // Process target-only files (not in source)
    let source_paths: std::collections::HashSet<_> = source_files
        .iter()
        .map(|f| f.relative_path.clone())
        .collect();
    for target in target_files {
        if !source_paths.contains(&target.relative_path) {
            // Target only - skip (don't delete)
            steps.push(MigrationStep {
                id: uuid::Uuid::new_v4().to_string(),
                source_path: String::new(),
                target_path: target.path.clone(),
                relative_path: target.relative_path.clone(),
                category: target.category.clone(),
                recommendation: Recommendation::Skip,
                status: MigrationStatus::Pending,
                reason: "Файл есть только в целевой версии.".to_string(),
            });
        }
    }

    let plan = MigrationPlan {
        id: uuid::Uuid::new_v4().to_string(),
        source_version: Version {
            major: 0,
            minor: 0,
            patch: 0,
            build: None,
            display_name: "Source".to_string(),
        },
        target_version: Version {
            major: 0,
            minor: 0,
            patch: 0,
            build: None,
            display_name: "Target".to_string(),
        },
        dry_run: true,
        total_files: source_files.len() + target_files.len(),
        conflicts,
        steps,
    };

    Ok(plan)
}

/// Execute a migration plan.
///
/// - `plan`: The migration plan to execute.
/// - `source_root`: Root directory where source files are located.
/// - `target_root`: Root directory where target files should be written.
/// - `backup_root`: Root directory where backups should be stored.
pub fn execute(
    plan: &MigrationPlan,
    source_root: &Path,
    target_root: &Path,
    backup_root: &Path,
) -> Result<MigrationResult, MigrationError> {
    let mut result = MigrationResult {
        completed_steps: 0,
        skipped_steps: 0,
        failed_steps: 0,
        errors: Vec::new(),
    };

    for step in &plan.steps {
        match step.recommendation {
            Recommendation::KeepSource => {
                // Copy from source to target using relative path
                let source_path = source_root.join(&step.relative_path);
                let target_path = target_root.join(&step.relative_path);

                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                if target_path.exists() {
                    create_backup(&target_path, backup_root, &plan.id)?;
                } else {
                    crate::backup::record_created_file(&target_path, backup_root, &plan.id)?;
                }
                fs::copy(&source_path, &target_path)?;
                result.completed_steps += 1;
            }
            Recommendation::KeepTarget => {
                // Do nothing - keep target as is
                result.skipped_steps += 1;
            }
            Recommendation::BackupAndKeepTarget => {
                let target_path = target_root.join(&step.relative_path);
                if target_path.exists() {
                    create_backup(&target_path, backup_root, &plan.id)?;
                }
                result.skipped_steps += 1;
            }
            Recommendation::BackupAndReplace => {
                // Create backup, then copy
                let target_path = target_root.join(&step.relative_path);
                if target_path.exists() {
                    create_backup(&target_path, backup_root, &plan.id)?;
                } else {
                    crate::backup::record_created_file(&target_path, backup_root, &plan.id)?;
                }
                let source_path = source_root.join(&step.relative_path);
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&source_path, &target_path)?;
                result.completed_steps += 1;
            }
            Recommendation::Skip => {
                // Do nothing
                result.skipped_steps += 1;
            }
            Recommendation::Merge => {
                // Not implemented yet - treat as conflict
                result.failed_steps += 1;
                result.errors.push(format!("Merge not supported for {}", step.target_path));
            }
        }
    }

    Ok(result)
}

/// Rollback a migration by restoring backed up files.
pub fn rollback(migration_id: &str, backup_root: &Path) -> Result<(), MigrationError> {
    let records = crate::backup::list_backups(backup_root)?;
    let migration_records: Vec<_> = records
        .into_iter()
        .filter(|r| r.migration_id == migration_id)
        .collect();

    if migration_records.is_empty() {
        return Err(MigrationError::NoRollbackData(migration_id.to_string()));
    }
    if migration_records.iter().all(|record| record.rolled_back) {
        return Err(MigrationError::AlreadyRolledBack(migration_id.to_string()));
    }

    for record in &migration_records {
        if record.rolled_back {
            continue;
        }
        if !record.created_by_migration && !Path::new(&record.backup_path).is_file() {
            return Err(MigrationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("backup file not found: {}", record.backup_path),
            )));
        }
    }

    for record in migration_records {
        if record.rolled_back {
            continue;
        }
        let original_path = Path::new(&record.original_path);
        if record.created_by_migration {
            if original_path.exists() {
                fs::remove_file(original_path)?;
            }
        } else {
            let backup_path = Path::new(&record.backup_path);
            if let Some(parent) = original_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(backup_path, original_path)?;
        }
    }

    crate::backup::mark_rolled_back(backup_root, migration_id)?;

    Ok(())
}

/// Save migration history entry to persistent storage.
pub fn save_migration_history(
    history_root: &Path,
    entry: &MigrationHistoryEntry,
) -> Result<(), MigrationError> {
    let index_path = history_root.join("migration_history.json");
    fs::create_dir_all(history_root)?;

    let mut entries = load_migration_history(history_root)?;
    entries.push(entry.clone());

    let json = serde_json::to_string_pretty(&entries)?;
    let temp_path = history_root.join("migration_history.json.tmp");
    fs::write(&temp_path, json)?;
    if index_path.exists() {
        fs::remove_file(&index_path)?;
    }
    fs::rename(temp_path, index_path)?;
    Ok(())
}

/// Load all migration history entries from persistent storage.
pub fn load_migration_history(
    history_root: &Path,
) -> Result<Vec<MigrationHistoryEntry>, MigrationError> {
    let index_path = history_root.join("migration_history.json");
    if !index_path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(index_path)?;
    let entries: Vec<MigrationHistoryEntry> = serde_json::from_str(&json)?;
    Ok(entries)
}

/// Clear all migration history entries.
pub fn clear_migration_history(
    history_root: &Path,
) -> Result<(), MigrationError> {
    let index_path = history_root.join("migration_history.json");
    if index_path.exists() {
        fs::remove_file(index_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, Classification, FileEntry};

    fn make_file(relative: &str, sha: &str) -> FileEntry {
        FileEntry {
            path: format!(r"C:\src\{relative}"),
            relative_path: relative.to_string(),
            category: Category::Scripts,
            size: 10,
            sha256: sha.to_string(),
            extension: "jsx".to_string(),
            last_modified: None,
            classification: Classification::default(),
        }
    }

    #[test]
    fn dry_run_identical_files() {
        let source = [make_file("scripts/a.jsx", "abc123")];
        let target = [make_file("scripts/a.jsx", "abc123")];
        let plan = dry_run(&source, &target).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].recommendation, Recommendation::KeepTarget);
        assert_eq!(plan.conflicts, 0);
    }

    #[test]
    fn dry_run_conflict() {
        let source = [make_file("scripts/a.jsx", "abc123")];
        let target = [make_file("scripts/a.jsx", "xyz789")];
        let plan = dry_run(&source, &target).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].recommendation, Recommendation::KeepTarget);
        assert_eq!(plan.conflicts, 1);
    }

    #[test]
    fn dry_run_source_only() {
        let source = [make_file("scripts/new.jsx", "abc123")];
        let target = [];
        let plan = dry_run(&source, &target).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].recommendation, Recommendation::KeepSource);
    }

    #[test]
    fn dry_run_target_only() {
        let source = [];
        let target = [make_file("scripts/old.jsx", "abc123")];
        let plan = dry_run(&source, &target).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].recommendation, Recommendation::Skip);
    }

    #[test]
    fn migration_history_persists_and_can_be_cleared() {
        let temp = tempfile::tempdir().unwrap();
        let entry = MigrationHistoryEntry {
            migration_id: "migration-123".to_string(),
            source_version: Version {
                major: 25,
                minor: 0,
                patch: 0,
                build: None,
                display_name: "After Effects 25".to_string(),
            },
            target_version: Version {
                major: 26,
                minor: 0,
                patch: 0,
                build: None,
                display_name: "After Effects 26".to_string(),
            },
            plan_id: "migration-123".to_string(),
            completed_steps: 3,
            skipped_steps: 1,
            failed_steps: 0,
            errors: Vec::new(),
            backup_root: "C:/backups".to_string(),
            timestamp: 1_700_000_000,
        };

        save_migration_history(temp.path(), &entry).unwrap();
        assert_eq!(load_migration_history(temp.path()).unwrap(), vec![entry]);

        clear_migration_history(temp.path()).unwrap();
        assert!(load_migration_history(temp.path()).unwrap().is_empty());
    }
}
