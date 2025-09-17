//! # Skeletor
//!
//! A blazing-fast Rust scaffolding tool with YAML-driven snapshots.
//!
//! Skeletor provides both a CLI interface and a programmatic API for:
//! - Creating file/directory structures from YAML configurations
//! - Taking snapshots of existing directory structures
//! - Extracting metadata from configuration files
//!
//! ## Usage as a Library
//!
//! ```no_run
//! use skeletor::SkeletorConfig;
//! use std::path::Path;
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new scaffolding configuration
//! let config = SkeletorConfig::from_yaml_str(r#"
//! directories:
//!   src:
//!     main.rs: |
//!       fn main() {
//!           println!("Hello, world!");
//!       }
//! "#)?;
//!
//! // Apply the configuration
//! let result = skeletor::apply_config(&config, Path::new("./my-project"), false, false)?;
//!
//! println!("Created {} files and {} directories", result.files_created, result.dirs_created);
//! # Ok(())
//! # }
//! ```

pub mod apply;
pub mod config;
pub mod errors;
pub mod info;
pub mod output;
pub mod snapshot;
pub mod tasks;

// Re-export key types for library users
pub use crate::config::{SkeletorConfig, SkeletorMetadata};
pub use crate::errors::SkeletorError;
pub use crate::output::{OutputFormat, Reporter, DefaultReporter, SilentReporter};

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Result of applying a configuration
#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub files_created: usize,
    pub dirs_created: usize,
    pub duration: Duration,
    pub tasks_total: usize,
}

/// Result of taking a directory snapshot
#[derive(Debug, Clone)]
pub struct SnapshotResult {
    pub files_processed: usize,
    pub dirs_processed: usize,
    pub duration: Duration,
    pub output_path: PathBuf,
    pub binary_files_excluded: usize,
}

/// Basic apply function for library usage
pub fn apply_config(
    config: &SkeletorConfig,
    target_dir: &Path,
    overwrite: bool,
    dry_run: bool,
) -> Result<ApplyResult, SkeletorError> {
    let start_time = Instant::now();
    let tasks = tasks::traverse_structure(target_dir, &config.directories);
    
    if dry_run {
        // For dry run, just return the task count
        Ok(ApplyResult {
            files_created: 0,
            dirs_created: 0,
            duration: start_time.elapsed(),
            tasks_total: tasks.len(),
        })
    } else {
        let (files_created, dirs_created) = 
            tasks::create_files_and_directories(&tasks, overwrite)?;
        
        Ok(ApplyResult {
            files_created,
            dirs_created,
            duration: start_time.elapsed(),
            tasks_total: tasks.len(),
        })
    }
}

// Note: Full snapshot library API implementation would require refactoring 
// the snapshot module to separate CLI concerns from core logic.
// For now, snapshot functionality is available through the CLI interface.