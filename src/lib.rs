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
pub mod utils;

#[cfg(test)]
pub mod test_utils;

// Re-export key types for library users
pub use crate::config::{SkeletorConfig, SkeletorMetadata};
pub use crate::errors::SkeletorError;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use clap::{Arg, ArgAction, Command};

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
    let tasks = tasks::traverse_structure(target_dir, &config.directories)?;
    
    if dry_run {
        // For dry run, just return the task count
        Ok(ApplyResult {
            files_created: 0,
            dirs_created: 0,
            duration: start_time.elapsed(),
            tasks_total: tasks.len(),
        })
    } else {
        let result = tasks::create_files_and_directories(&tasks, overwrite)?;
        
        Ok(ApplyResult {
            files_created: result.files_created,
            dirs_created: result.dirs_created,
            duration: start_time.elapsed(),
            tasks_total: tasks.len(),
        })
    }
}

// Note: Full snapshot library API implementation would require refactoring 
// the snapshot module to separate CLI concerns from core logic.
// For now, snapshot functionality is available through the CLI interface.

/// Build the CLI interface with three subcommands: `apply`, `snapshot` and `info`
/// This function is used by both the main CLI and by tests to ensure consistency
pub fn build_cli() -> Command {
    Command::new("Skeletor")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Jason Joseph Nathan")
        .about("A blazing-fast Rust scaffolding tool with snapshot capabilities.\n\nSkeletor helps you create project templates and scaffold new projects from YAML configurations.\nYou can capture existing folder structures as templates and apply them to create new projects.\n\nCommon workflow:\n  1. skeletor snapshot my-project -o template.yml  # Capture existing project\n  2. skeletor apply template.yml                   # Apply template elsewhere")
        .subcommand_required(true)
        .subcommand(
            Command::new("apply")
                .about("Creates files and directories based on a YAML configuration\n\nEXAMPLES:\n  skeletor apply                           # Use .skeletorrc config in current dir\n  skeletor apply my-template.yml           # Use custom config in current dir\n  skeletor apply -o ../new-project         # Apply to different directory\n  skeletor apply --dry-run                 # Preview changes (summary)\n  skeletor apply --dry-run --verbose       # Preview changes (full listing)")
                .arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("YAML configuration file (default: .skeletorrc)")
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("DIR")
                        .help("Output directory where files will be created (default: current directory)"),
                )
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .help("Overwrite existing files if they already exist (default: off)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry_run")
                        .short('d')
                        .long("dry-run")
                        .help("Preview changes without writing files (default: off; summary by default)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show full operation listing (default: off; affects dry-run and apply output)")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Creates a .skeletorrc snapshot from an existing folder\n\nEXAMPLES:\n  skeletor snapshot my-project               # Write .skeletorrc\n  skeletor snapshot my-project -o config.yml # Save to file\n  skeletor snapshot my-project --stdout      # Print YAML to stdout\n  skeletor snapshot src/ -i \"*.log\" -i target/ # Ignore build artifacts\n  skeletor snapshot --dry-run my-project     # Preview snapshot (summary)\n  skeletor snapshot --dry-run --verbose my-project # Preview with details\n\nIMPORTANT: Quote glob patterns to prevent shell expansion:\n  ✓ skeletor snapshot -i \"*.log\" -i \"src/**/*.tmp\" .\n  ✗ skeletor snapshot -i *.log -i src/**/*.tmp .  # Shell expands patterns")
                .arg(
                    Arg::new("source")
                        .value_name("FOLDER")
                        .help("The source folder to snapshot")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Save snapshot YAML to a file (default: .skeletorrc)"),
                )
                .arg(
                    Arg::new("stdout")
                        .long("stdout")
                        .help("Print snapshot YAML to stdout instead of writing to a file (default: write to file)")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("output"),
                )
                .arg(
                    Arg::new("exclude_contents")
                        .long("exclude-contents")
                        .help("Exclude file contents (default: include contents; binary files still detected)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("ignore")
                        .short('i')
                        .long("ignore")
                        .value_name("PATTERN_OR_FILE")
                        .help("Exclude files from snapshot (default: none; can be used multiple times)\n  • Patterns: \"*.log\", \"target/*\", \"node_modules/\" (QUOTE THEM!)\n  • Files: \".gitignore\", \".dockerignore\" (auto-detected)")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("ignore_file")
                        .long("ignore-file")
                        .value_name("FILE")
                        .help("Read ignore patterns from a file (default: none; use multiple times)")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed matching and file processing info (default: off)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry_run")
                        .short('d')
                        .long("dry-run")
                        .help("Preview snapshot without writing files (default: off; summary by default)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("note")
                        .short('n')
                        .long("note")
                        .value_name("NOTE")
                        .help("Attach a user-defined note to the snapshot (default: none)"),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("Displays metadata from a .skeletorrc file\n\nEXAMPLES:\n  skeletor info                             # Show info for .skeletorrc\n  skeletor info my-template.yml             # Show info for custom file")
                .arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("YAML configuration file to inspect (default: .skeletorrc)")
                        .index(1),
                ),
        )
}
