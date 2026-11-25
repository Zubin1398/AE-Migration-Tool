pub mod detection;
pub mod models;
pub mod scanner;
pub mod migration;
pub mod backup;

// Re-export public types
pub use migration::{dry_run, execute, rollback, MigrationError};
pub use models::{MigrationPlan, MigrationResult};