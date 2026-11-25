use serde::{Deserialize, Serialize};

// Shared data structures for the AE Migration Tool core.

/// Detected After Effects installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installation {
    pub id: String,
    pub version: Version,
    pub install_path: String,
    pub support_files_path: String,
    pub detection_method: DetectionMethod,
    pub is_complete: bool,
}

/// After Effects version number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<u32>,
    pub display_name: String,
}

/// How the installation was discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    Filesystem,
    Registry,
    Manual,
}

/// A file discovered during scanning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub relative_path: String,
    pub category: Category,
    pub size: u64,
    pub sha256: String,
    pub extension: String,
    pub last_modified: Option<u64>,
    pub classification: Classification,
}

/// Migration category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Plugins,
    Scripts,
    ScriptUIPanels,
    CEPExtensions,
    Presets,
    Workspaces,
    KeyboardShortcuts,
    Preferences,
    SharedResource,
    Unknown,
}

/// Classification result for a scanned file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    pub category: Category,
    pub confidence: Confidence,
}

impl Default for Classification {
    fn default() -> Self {
        Self {
            category: Category::Unknown,
            confidence: Confidence::Low,
        }
    }
}

/// Confidence level for analysis results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
    Unsafe,
}

/// Recommendation for a conflicting file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    KeepTarget,
    BackupAndKeepTarget,
    KeepSource,
    Merge,
    Skip,
    BackupAndReplace,
}

/// Status of a migration operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
    Cancelled,
}

/// A single planned migration step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStep {
    pub id: String,
    pub source_path: String,
    pub target_path: String,
    pub relative_path: String,
    pub category: Category,
    pub recommendation: Recommendation,
    pub status: MigrationStatus,
    pub reason: String,
}

/// Complete migration plan (Dry Run result).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: String,
    pub source_version: Version,
    pub target_version: Version,
    pub dry_run: bool,
    pub steps: Vec<MigrationStep>,
    pub total_files: usize,
    pub conflicts: usize,
}

/// Result of migration execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationResult {
    pub completed_steps: usize,
    pub skipped_steps: usize,
    pub failed_steps: usize,
    pub errors: Vec<String>,
}

/// Record of a created backup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub migration_id: String,
    pub original_path: String,
    pub backup_path: String,
    #[serde(default)]
    pub created_by_migration: bool,
    #[serde(default)]
    pub rolled_back: bool,
    pub created_at: u64,
}

/// Migration history entry for persistent storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationHistoryEntry {
    pub migration_id: String,
    pub source_version: Version,
    pub target_version: Version,
    pub plan_id: String,
    pub completed_steps: usize,
    pub skipped_steps: usize,
    pub failed_steps: usize,
    pub errors: Vec<String>,
    pub backup_root: String,
    pub timestamp: u64,
}

/// Journal entry for transactional rollback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub migration_id: String,
    pub step_id: String,
    pub action: String,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
    pub completed: bool,
    pub timestamp: u64,
}
