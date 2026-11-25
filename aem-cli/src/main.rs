use aem_core::{
    detection::detect_installations,
    migration::{dry_run, execute, rollback},
    scanner::scan_directory,
    backup::list_backups,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aem-migration", version, about = "After Effects Migration Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect After Effects installations
    Detect,
    /// Scan a directory for migration files
    Scan {
        /// Directory to scan
        path: PathBuf,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Generate a dry-run migration plan
    Plan {
        /// Source installation path
        source: PathBuf,
        /// Target installation path
        target: PathBuf,
        /// Output plan as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Execute a migration plan
    Execute {
        /// Source installation path
        source: PathBuf,
        /// Target installation path
        target: PathBuf,
        /// Backup directory
        #[arg(short, long)]
        backup: Option<PathBuf>,
    },
    /// Rollback a migration
    Rollback {
        /// Migration plan ID
        plan_id: String,
        /// Backup directory
        #[arg(short, long)]
        backup: PathBuf,
    },
    /// List backups
    ListBackups {
        /// Backup directory
        backup: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Detect => {
            let installations = detect_installations()?;
            if installations.is_empty() {
                println!("No After Effects installations found.");
            } else {
                for inst in &installations {
                    println!("{:?}", inst);
                }
            }
        }
        Commands::Scan { path, json } => {
            let entries = scan_directory(&path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                for entry in entries {
                    println!("{} ({:?}) - {} bytes", entry.relative_path, entry.category, entry.size);
                }
            }
        }
        Commands::Plan { source, target, json } => {
            let source_files = scan_directory(&source)?;
            let target_files = scan_directory(&target)?;

            let plan = dry_run(&source_files, &target_files)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("Migration Plan: {}", plan.id);
                println!("Source: {:?} -> Target: {:?}", plan.source_version, plan.target_version);
                println!("Total files: {}, Conflicts: {}", plan.total_files, plan.conflicts);
                for step in &plan.steps {
                    println!("  [{:?}] {} -> {}", step.recommendation, step.relative_path, step.reason);
                }
            }
        }
        Commands::Execute { source, target, backup } => {
            let source_files = scan_directory(&source)?;
            let target_files = scan_directory(&target)?;

            let plan = dry_run(&source_files, &target_files)?;

            let backup_root = backup.unwrap_or_else(|| target.join("..").join("AE-Migration-Tool-Backups"));

            let result = execute(&plan, &source, &target, &backup_root)?;

            println!("Migration ID: {}", plan.id);
            println!("Migration completed: {} steps, {} skipped, {} failed",
                result.completed_steps, result.skipped_steps, result.failed_steps);
            if !result.errors.is_empty() {
                for err in &result.errors {
                    eprintln!("Error: {}", err);
                }
            }
        }
        Commands::Rollback { plan_id, backup } => {
            rollback(&plan_id, &backup)?;
            println!("Rollback completed for plan: {}", plan_id);
        }
        Commands::ListBackups { backup } => {
            let records = list_backups(&backup)?;
            if records.is_empty() {
                println!("No backups found.");
            } else {
                for record in records {
                    println!("{} | {} | {}", record.id, record.migration_id, record.original_path);
                }
            }
        }
    }

    Ok(())
}
