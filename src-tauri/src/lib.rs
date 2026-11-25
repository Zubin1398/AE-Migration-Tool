use aem_core::{
    models::{Category, MigrationHistoryEntry, MigrationPlan, MigrationResult},
    scanner::{scan_directory, scan_directory_filtered},
};
use tauri::{command, State, Manager};
use std::sync::Mutex;
use std::path::PathBuf;

fn history_root() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AE-Migration-Tool")
}

#[derive(Default)]
struct AppState {
    last_plan: Mutex<Option<MigrationPlan>>,
}

#[command]
fn detect_installations_cmd() -> Result<Vec<aem_core::models::Installation>, String> {
    aem_core::detection::detect_installations().map_err(|e| e.to_string())
}

#[command]
fn add_manual_installation(path: String) -> Result<aem_core::models::Installation, String> {
    aem_core::detection::add_manual(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[command]
fn scan_directory_cmd(path: String, categories: Option<Vec<Category>>) -> Result<Vec<aem_core::models::FileEntry>, String> {
    if let Some(cats) = categories {
        aem_core::scanner::scan_directory_filtered(&PathBuf::from(path), &cats).map_err(|e| e.to_string())
    } else {
        aem_core::scanner::scan_directory(&PathBuf::from(path)).map_err(|e| e.to_string())
    }
}

#[command]
fn create_migration_plan(
    source_path: String,
    target_path: String,
    source_version: Option<aem_core::models::Version>,
    target_version: Option<aem_core::models::Version>,
    categories: Option<Vec<Category>>,
    state: State<'_, AppState>,
) -> Result<MigrationPlan, String> {
    let source_root = PathBuf::from(&source_path);
    let target_root = PathBuf::from(&target_path);
    let source_files = match &categories {
        Some(categories) => scan_directory_filtered(&source_root, categories),
        None => scan_directory(&source_root),
    }.map_err(|e| e.to_string())?;
    let target_files = match &categories {
        Some(categories) => scan_directory_filtered(&target_root, categories),
        None => scan_directory(&target_root),
    }.map_err(|e| e.to_string())?;
    
    let mut plan = aem_core::migration::dry_run(&source_files, &target_files).map_err(|e| e.to_string())?;
    if let Some(version) = source_version {
        plan.source_version = version;
    }
    if let Some(version) = target_version {
        plan.target_version = version;
    }
    
    *state.last_plan.lock().unwrap() = Some(plan.clone());
    
    Ok(plan)
}

#[command]
fn execute_migration(
    plan: MigrationPlan,
    source_path: String,
    target_path: String,
    backup_path: Option<String>,
) -> Result<MigrationResult, String> {
    let backup_root = backup_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&target_path).join("..").join("AE-Migration-Tool-Backups"));
    
    let result = aem_core::migration::execute(&plan, &PathBuf::from(source_path), &PathBuf::from(target_path), &backup_root)
        .map_err(|e| e.to_string())?;
    
    // Save migration history
    let history_entry = MigrationHistoryEntry {
        migration_id: plan.id.clone(),
        source_version: plan.source_version.clone(),
        target_version: plan.target_version.clone(),
        plan_id: plan.id.clone(),
        completed_steps: result.completed_steps,
        skipped_steps: result.skipped_steps,
        failed_steps: result.failed_steps,
        errors: result.errors.clone(),
        backup_root: backup_root.to_string_lossy().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    aem_core::migration::save_migration_history(&history_root(), &history_entry).map_err(|e| e.to_string())?;
    
    Ok(result)
}

#[command]
fn rollback_migration(plan_id: String, backup_path: String) -> Result<(), String> {
    aem_core::migration::rollback(&plan_id, &PathBuf::from(backup_path)).map_err(|e| e.to_string())
}

#[command]
fn list_backups_cmd(backup_path: String) -> Result<Vec<aem_core::models::BackupRecord>, String> {
    aem_core::backup::list_backups(&PathBuf::from(backup_path)).map_err(|e| e.to_string())
}

#[command]
fn get_migration_history(installation_paths: Option<Vec<String>>) -> Result<Vec<MigrationHistoryEntry>, String> {
    let mut entries = aem_core::migration::load_migration_history(&history_root())
        .map_err(|error| error.to_string())?;
    let mut known_ids: std::collections::HashSet<String> = entries
        .iter()
        .map(|entry| entry.migration_id.clone())
        .collect();

    // Older builds stored history alongside backups next to each target
    // installation. Read those indexes so existing migrations remain visible.
    for install_path in installation_paths.unwrap_or_default() {
        let Some(parent) = PathBuf::from(install_path).parent().map(PathBuf::from) else {
            continue;
        };
        let legacy_root = parent.join("AE-Migration-Tool-Backups");
        for entry in aem_core::migration::load_migration_history(&legacy_root)
            .map_err(|error| error.to_string())?
        {
            if known_ids.insert(entry.migration_id.clone()) {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

#[command]
fn clear_migration_history() -> Result<(), String> {
    aem_core::migration::clear_migration_history(&history_root()).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            detect_installations_cmd,
            add_manual_installation,
            scan_directory_cmd,
            create_migration_plan,
            execute_migration,
            rollback_migration,
            list_backups_cmd,
            get_migration_history,
            clear_migration_history,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
