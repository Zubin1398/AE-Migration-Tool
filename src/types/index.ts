export interface Installation {
  id: string;
  version: Version;
  install_path: string;
  support_files_path: string;
  detection_method: 'Filesystem' | 'Registry' | 'Manual';
  is_complete: boolean;
}

export interface Version {
  major: number;
  minor: number;
  patch: number;
  build: number | null;
  display_name: string;
}

export interface FileEntry {
  path: string;
  relative_path: string;
  category: Category;
  size: number;
  sha256: string;
  extension: string;
  last_modified: number | null;
  classification: Classification;
}

export type Category =
  | 'Plugins'
  | 'Scripts'
  | 'ScriptUIPanels'
  | 'CEPExtensions'
  | 'Presets'
  | 'Workspaces'
  | 'KeyboardShortcuts'
  | 'Preferences'
  | 'SharedResource'
  | 'Unknown';

export interface Classification {
  category: Category;
  confidence: 'High' | 'Medium' | 'Low' | 'Unsafe';
}

export interface MigrationStep {
  id: string;
  source_path: string;
  target_path: string;
  relative_path: string;
  category: Category;
  recommendation: Recommendation;
  status: MigrationStatus;
  reason: string;
}

export type Recommendation =
  | 'KeepTarget'
  | 'BackupAndKeepTarget'
  | 'KeepSource'
  | 'Merge'
  | 'Skip'
  | 'BackupAndReplace';

export type MigrationStatus =
  | 'Pending'
  | 'Running'
  | 'Completed'
  | 'Failed'
  | 'RolledBack'
  | 'Cancelled';

export interface MigrationPlan {
  id: string;
  source_version: Version;
  target_version: Version;
  dry_run: boolean;
  steps: MigrationStep[];
  total_files: number;
  conflicts: number;
}

export interface MigrationResult {
  completed_steps: number;
  skipped_steps: number;
  failed_steps: number;
  errors: string[];
}

export interface BackupRecord {
  id: string;
  migration_id: string;
  original_path: string;
  backup_path: string;
  created_by_migration: boolean;
  rolled_back: boolean;
  created_at: number;
}

export interface MigrationHistoryEntry {
  migration_id: string;
  source_version: Version;
  target_version: Version;
  plan_id: string;
  completed_steps: number;
  skipped_steps: number;
  failed_steps: number;
  errors: string[];
  backup_root: string;
  timestamp: number;
}

export type Page =
  | 'dashboard'
  | 'version-selection'
  | 'migration-options'
  | 'scan-analysis'
  | 'migration-report'
  | 'migration'
  | 'history';

export interface CategoryOption {
  key: Category;
  label: string;
  hint: string;
  icon: React.ReactNode;
}
