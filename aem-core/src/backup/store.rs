use crate::models::BackupRecord;
use crate::backup::BackupError;
use serde_json;
use std::fs;
use std::path::Path;

/// Path to the backup records index file.
fn index_path(backup_root: &Path) -> std::path::PathBuf {
    backup_root.join("backup_index.json")
}

/// Save a backup record to the index.
pub fn save_record(backup_root: &Path, record: &BackupRecord) -> Result<(), BackupError> {
    let index = index_path(backup_root);
    fs::create_dir_all(backup_root)?;

    let mut records = load_all_records(backup_root)?;
    records.push(record.clone());

    let json = serde_json::to_string_pretty(&records)?;
    fs::write(index, json)?;
    Ok(())
}

/// Replace the backup index with the supplied records.
pub fn save_all_records(backup_root: &Path, records: &[BackupRecord]) -> Result<(), BackupError> {
    fs::create_dir_all(backup_root)?;
    let json = serde_json::to_string_pretty(records)?;
    fs::write(index_path(backup_root), json)?;
    Ok(())
}

/// Load all backup records from the index.
pub fn load_all_records(backup_root: &Path) -> Result<Vec<BackupRecord>, BackupError> {
    let index = index_path(backup_root);
    if !index.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(index)?;
    let records: Vec<BackupRecord> = serde_json::from_str(&json)?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BackupRecord;
    use tempfile::tempdir;

    #[test]
    fn save_and_load_records() {
        let temp = tempdir().unwrap();
        let backup_root = temp.path().join("backups");

        let record = BackupRecord {
            id: "test-1".to_string(),
            migration_id: "migration-123".to_string(),
            original_path: r"C:\source.txt".to_string(),
            backup_path: r"C:\backups\migration-123\source.txt".to_string(),
            created_by_migration: false,
            rolled_back: false,
            created_at: 1234567890,
        };

        save_record(&backup_root, &record).unwrap();
        let records = load_all_records(&backup_root).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "test-1");
    }

    #[test]
    fn load_empty_returns_empty() {
        let temp = tempdir().unwrap();
        let backup_root = temp.path().join("backups");
        let records = load_all_records(&backup_root).unwrap();
        assert!(records.is_empty());
    }
}
