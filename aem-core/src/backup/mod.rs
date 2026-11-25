pub mod store;

use crate::models::BackupRecord;
use thiserror::Error;
use std::fs;
use std::path::Path;

/// Error during backup operations.
#[derive(Debug, Error)]
pub enum BackupError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("source file does not exist: {0}")]
    SourceNotFound(String),
    #[error("backup verification failed: {0}")]
    VerificationFailed(String),
}

/// Create a backup of a file before modification.
///
/// Returns the backup ID.
pub fn create_backup(
    source_path: &std::path::Path,
    backup_root: &std::path::Path,
    migration_id: &str,
) -> Result<String, BackupError> {
    if !source_path.exists() {
        return Err(BackupError::SourceNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    let backup_id = uuid::Uuid::new_v4().to_string();
    let migration_dir = backup_root.join(migration_id);

    // Keep the original name when possible, but make colliding file names unique.
    let relative_path = source_path
        .file_name()
        .ok_or_else(|| BackupError::SourceNotFound("invalid source path".to_string()))?;

    fs::create_dir_all(&migration_dir)?;

    let mut backup_path = migration_dir.join(relative_path);
    if backup_path.exists() {
        backup_path = migration_dir.join(format!("{backup_id}-{}", relative_path.to_string_lossy()));
    }
    fs::copy(source_path, &backup_path)?;

    // Create backup record
    let record = BackupRecord {
        id: backup_id.clone(),
        migration_id: migration_id.to_string(),
        original_path: source_path.to_string_lossy().to_string(),
        backup_path: backup_path.to_string_lossy().to_string(),
        created_by_migration: false,
        rolled_back: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Save record
    store::save_record(&backup_root, &record)?;

    Ok(backup_id)
}

/// Record a new target file so rollback can remove it after reversing a migration.
pub fn record_created_file(
    target_path: &std::path::Path,
    backup_root: &std::path::Path,
    migration_id: &str,
) -> Result<(), BackupError> {
    let record = BackupRecord {
        id: uuid::Uuid::new_v4().to_string(),
        migration_id: migration_id.to_string(),
        original_path: target_path.to_string_lossy().to_string(),
        backup_path: String::new(),
        created_by_migration: true,
        rolled_back: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    store::save_record(backup_root, &record)?;
    Ok(())
}

/// List all backup records in the backup root.
pub fn list_backups(backup_root: &std::path::Path) -> Result<Vec<BackupRecord>, BackupError> {
    store::load_all_records(backup_root)
}

/// Persist the completed rollback state for every record in a migration.
pub fn mark_rolled_back(backup_root: &Path, migration_id: &str) -> Result<(), BackupError> {
    let mut records = store::load_all_records(backup_root)?;
    for record in &mut records {
        if record.migration_id == migration_id {
            record.rolled_back = true;
        }
    }
    store::save_all_records(backup_root, &records)
}

/// Verify the integrity of a backup by comparing SHA-256 hashes.
pub fn verify_integrity(record: &BackupRecord) -> Result<bool, BackupError> {
    let source_hash = compute_sha256(&record.original_path)?;
    let backup_hash = compute_sha256(&record.backup_path)?;
    Ok(source_hash == backup_hash)
}

fn compute_sha256(path: &str) -> Result<String, BackupError> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn create_backup_creates_copy() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source.txt");
        let backup_root = temp.path().join("backups");
        fs::write(&source, "hello world").unwrap();

        let backup_id = create_backup(&source, &backup_root, "migration-123").unwrap();
        assert!(!backup_id.is_empty());

        let backup_path = backup_root.join("migration-123").join("source.txt");
        assert!(backup_path.exists());
        assert_eq!(fs::read_to_string(&backup_path).unwrap(), "hello world");
    }

    #[test]
    fn list_backups_returns_records() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source.txt");
        let backup_root = temp.path().join("backups");
        fs::write(&source, "hello").unwrap();

        create_backup(&source, &backup_root, "migration-123").unwrap();
        let records = list_backups(&backup_root).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].migration_id, "migration-123");
    }

    #[test]
    fn verify_integrity_passes_for_valid_backup() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source.txt");
        let backup_root = temp.path().join("backups");
        fs::write(&source, "hello world").unwrap();

        create_backup(&source, &backup_root, "migration-123").unwrap();
        let records = list_backups(&backup_root).unwrap();
        let ok = verify_integrity(&records[0]).unwrap();
        assert!(ok);
    }

    #[test]
    fn create_backup_fails_for_nonexistent() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("nonexistent.txt");
        let backup_root = temp.path().join("backups");
        let result = create_backup(&source, &backup_root, "migration-123");
        assert!(result.is_err());
    }
}
