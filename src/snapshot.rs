use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::output::{DefaultReporter, SimpleSnapshotResult, Reporter};
use crate::tasks::{compute_stats, traverse_directory, Task};
use chrono::Utc;
use clap::ArgMatches;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::info;
use serde_yaml::{Mapping, Value};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Configuration for snapshot command extracted from CLI arguments
struct SnapshotConfig {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub include_contents: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub user_note: Option<String>,
}

impl SnapshotConfig {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            source_path: PathBuf::from(matches.get_one::<String>("source").unwrap()),
            output_path: default_file_path(matches.get_one::<String>("output")),
            include_contents: matches.get_flag("include_contents"),
            dry_run: matches.get_flag("dry_run"),
            verbose: matches.get_flag("verbose"),
            user_note: matches.get_one::<String>("note").map(|s| s.to_string()),
        }
    }
}

/// Handles verbose information collection and display
fn prepare_verbose_info(ignore_patterns: &[String], verbose: bool) -> Vec<String> {
    let mut verbose_info = Vec::new();
    if verbose {
        verbose_info.push(format!("Loaded ignore patterns: {:?}", ignore_patterns));
        if !ignore_patterns.is_empty() {
            for pattern in ignore_patterns {
                verbose_info.push(format!("Added ignore pattern: {}", pattern));
            }
        }
    } else if !ignore_patterns.is_empty() {
        // Add ignore pattern count to verbose info for non-verbose mode
        verbose_info.push(format!("Using {} ignore pattern(s)", ignore_patterns.len()));
    }
    verbose_info
}

/// Runs the snapshot subcommand: Generates a structured snapshot and writes it to disk.
pub fn run_snapshot(matches: &ArgMatches) -> Result<(), SkeletorError> {
    let config = SnapshotConfig::from_matches(matches);
    
    info!("Taking snapshot of folder: {:?}", config.source_path);
    let start_time = Instant::now();

    // Process ignore patterns and prepare verbose information
    let ignore_patterns = collect_ignore_patterns(matches)?;
    let verbose_info = prepare_verbose_info(&ignore_patterns, config.verbose);

    // Build globset and take snapshot
    let globset = build_globset(&ignore_patterns, false)?;
    let (dir_snapshot, binary_files) = traverse_directory(
        &config.source_path, 
        config.include_contents, 
        globset.as_ref(), 
        false
    )?;
    let (files_count, dirs_count) = compute_stats(&dir_snapshot);

    // Build and write snapshot
    let snapshot = build_snapshot(
        &config.output_path,
        config.user_note,
        dir_snapshot.clone(), // Clone to avoid borrow issues with dry-run
        binary_files.clone(),
        files_count,
        dirs_count,
    )?;

    let duration = start_time.elapsed();
    let reporter = DefaultReporter::new();
    
    if config.dry_run {
        // Use Reporter system for consistent dry-run formatting
        display_snapshot_dry_run_comprehensive(&dir_snapshot, config.verbose, &binary_files, &ignore_patterns)?;
    } else {
        write_snapshot_with_reporter(snapshot, &config.output_path, verbose_info)?;
        
        let snapshot_result = SimpleSnapshotResult {
            files_processed: files_count,
            dirs_processed: dirs_count,
            duration,
            output_path: config.output_path,
            binary_files_excluded: binary_files.len(),
            binary_files_list: binary_files,
        };
        reporter.snapshot_complete(&snapshot_result);
    }
    
    Ok(())
}

/// Collects ignore patterns from CLI arguments.
fn collect_ignore_patterns(matches: &ArgMatches) -> Result<Vec<String>, SkeletorError> {
    let mut ignore_patterns = Vec::new();

    if let Some(vals) = matches.get_many::<String>("ignore") {
        for val in vals {
            let candidate = Path::new(val);
            if candidate.exists() && candidate.is_file() {
                // Read file (e.g., `.gitignore`) and add valid patterns
                let content = crate::utils::read_file_to_string(candidate)?;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        ignore_patterns.push(trimmed.to_string());
                    }
                }
            } else {
                // Treat as a direct glob pattern.
                ignore_patterns.push(val.to_string());
            }
        }
    }
    Ok(ignore_patterns)
}

fn build_globset(ignore_patterns: &[String], _verbose: bool) -> Result<Option<GlobSet>, SkeletorError> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pat in ignore_patterns {
        let normalized_pattern = pat.trim().to_string();
        match Glob::new(&normalized_pattern) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                return Err(SkeletorError::InvalidIgnorePattern { 
                    pattern: format!("{} ({})", normalized_pattern, e) 
                });
            }
        }
    }

    builder
        .build()
        .map(Some)
        .map_err(|e| SkeletorError::InvalidIgnorePattern { 
            pattern: format!("Failed to compile ignore patterns: {}", e) 
        })
}

/// Builds a structured snapshot with metadata.
fn build_snapshot(
    output_path: &Path,
    user_note: Option<String>,
    dir_snapshot: Value,
    binary_files: Vec<String>,
    files_count: usize,
    dirs_count: usize,
) -> Result<Value, SkeletorError> {
    let now = Utc::now().to_rfc3339();
    let mut created = now.clone();

    // Preserve "created" timestamp if output file exists
    if output_path.exists() {
        if let Ok(existing_config) = read_config(output_path) {
            if let Some(Value::String(c)) = existing_config.get("created") {
                created = c.clone();
            }
        }
    }

    let updated = now;

    let mut auto_info = format!("Snapshot generated from folder: {:?}", output_path);
    if binary_files.is_empty() {
        auto_info.push_str("\nNo binary files detected.");
    } else {
        auto_info.push_str(&format!(
            "\nBinary files detected (contents omitted): {:?}",
            binary_files
        ));
    }

    let mut top_map = Mapping::new();
    top_map.insert(Value::String("created".to_string()), Value::String(created));
    top_map.insert(Value::String("updated".to_string()), Value::String(updated));
    top_map.insert(
        Value::String("generated_comments".to_string()),
        Value::String(auto_info),
    );

    if let Some(note) = user_note {
        top_map.insert(Value::String("notes".to_string()), Value::String(note));
    }

    let mut stats_map = Mapping::new();
    stats_map.insert(
        Value::String("files".to_string()),
        Value::Number(files_count.into()),
    );
    stats_map.insert(
        Value::String("directories".to_string()),
        Value::Number(dirs_count.into()),
    );

    top_map.insert(
        Value::String("stats".to_string()),
        Value::Mapping(stats_map),
    );
    top_map.insert(Value::String("directories".to_string()), dir_snapshot);

    Ok(Value::Mapping(top_map))
}

/// Displays snapshot dry run output using professional formatting
#[allow(dead_code)]
fn display_snapshot_dry_run(snapshot: &Value, verbose_info: Vec<String>) -> Result<(), SkeletorError> {
    let out_str = serde_yaml::to_string(snapshot).map_err(|e| SkeletorError::Config(e.to_string()))?;
    
    // Simple, clean dry run output like v0.3.1
    println!("Dry run enabled. The following snapshot would be generated:");
    println!("{}", out_str);
    
    // Display verbose information if available
    if !verbose_info.is_empty() {
        for info in verbose_info {
            println!("{}", info);
        }
    }
    
    Ok(())
}

/// Convert snapshot directory structure to list of operations (tasks)
fn snapshot_to_operations(dir_snapshot: &Value, base_path: &str) -> Vec<Task> {
    let mut operations = Vec::new();
    
    if let Some(mapping) = dir_snapshot.as_mapping() {
        for (key, value) in mapping {
            if let Some(name) = key.as_str() {
                let path = if base_path.is_empty() {
                    format!("./{}", name)
                } else {
                    format!("{}/{}", base_path, name)
                };
                
                if value.as_mapping().is_some() {
                    // This is a directory
                    operations.push(Task::Dir(path.clone().into()));
                    // Recursively process subdirectories and files
                    operations.extend(snapshot_to_operations(value, &path));
                } else if let Some(_content) = value.as_str() {
                    // This is a file
                    operations.push(Task::File(path.into(), "".to_string()));
                }
            }
        }
    }
    
    operations
}

/// Displays comprehensive snapshot dry run using Reporter system for consistency
fn display_snapshot_dry_run_comprehensive(
    dir_snapshot: &Value, 
    verbose: bool, 
    binary_files: &[String], 
    ignore_patterns: &[String]
) -> Result<(), SkeletorError> {
    // Convert snapshot structure to operations for consistent display
    let operations = snapshot_to_operations(dir_snapshot, "");
    
    // Use the Reporter system for consistent formatting
    let reporter = DefaultReporter::new();
    reporter.dry_run_preview_comprehensive(&operations, verbose, binary_files, ignore_patterns, "captured");
    
    Ok(())
}

/// Writes snapshot to disk - output handled by Reporter system
fn write_snapshot_with_reporter(snapshot: Value, output_path: &Path, verbose_info: Vec<String>) -> Result<(), SkeletorError> {
    let out_str = serde_yaml::to_string(&snapshot).map_err(|e| SkeletorError::Config(e.to_string()))?;
    
    crate::utils::write_string_to_file(output_path, &out_str)?;
    
    // Verbose information display (if needed)
    if !verbose_info.is_empty() {
        for info in verbose_info {
            println!("{}", info);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::*;
    use crate::test_utils::helpers::*;

    #[test]
    fn test_snapshot_directory_without_contents() {
        let fs = TestFileSystem::new();

        // Create a simple structure with a hidden file and a regular file.
        fs.create_file("src/index.js", "console.log('Hello');");
        // Hidden file should be included.
        fs.create_file("src/.hidden.txt", "secret");

        let (yaml_structure, binaries) = traverse_directory(&fs.root_path, false, None, false).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(Value::String("src".to_string())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_snapshot_directory_with_contents() {
        let fs = TestFileSystem::new();

        // Create a simple structure with a hidden file and a regular file.
        fs.create_file("src/index.js", "console.log('Hello');");
        // Hidden file should be included.
        fs.create_file("src/.hidden.txt", "secret");

        let (yaml_structure, binaries) = traverse_directory(&fs.root_path, true, None, false).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(Value::String("src".to_string())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_compute_stats() {
        let yaml_str = r#"
        src:
          index.js: "console.log('Hello, world!');"
          components:
            Header.js: "// Header component"
        "#;

        // 🔥 Corrected: Properly handle the Result
        let yaml: Value = serde_yaml::from_str(yaml_str).expect("Failed to parse YAML");

        let (files, dirs) = compute_stats(&yaml);

        assert_eq!(files, 2);
        assert_eq!(dirs, 2); // One for "src" and one for "components"
    }

    #[test]
    fn test_traverse_directory() {
        let fs = TestFileSystem::new();

        // Create a simple structure with a hidden file and a regular file.
        fs.create_file("src/index.js", "console.log('Hello');");
        // Hidden file should be included.
        fs.create_file("src/.hidden.txt", "secret");

        let (yaml_structure, binaries) = traverse_directory(&fs.root_path, false, None, false).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(Value::String("src".to_string())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_run_snapshot_with_dry_run() {
        let fs = TestFileSystem::new();
        

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");

        let args = vec![&fs.root_path.to_str().unwrap(), "--dry-run"];

        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_output() {
        let fs = TestFileSystem::new();
        
        let output_file = &fs.root_path.join("output.yaml");

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");

        let args = vec![
            &fs.root_path.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ];

        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok());
            assert!(output_file.exists());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_ignore_patterns() {
        let fs = TestFileSystem::new();
        

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");
        fs.create_file("src/ignore.txt", "ignore me");

        let ignore_file = fs.create_file("ignore_patterns.txt", "ignore.txt");

        let args = vec![
            &fs.root_path.to_str().unwrap(),
            "--ignore",
            ignore_file.to_str().unwrap(),
        ];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
    #[test]
    fn test_run_snapshot_with_binary_files() {
        let fs = TestFileSystem::new();
        

        // Create a simple structure with a binary file.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");
        fs.create_binary_file("src/binary.bin", &[0, 159, 146, 150]);

        let args = vec![fs.root_path.to_str().unwrap()];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
    #[test]
    fn test_run_snapshot_with_notes() {
        let fs = TestFileSystem::new();
        
        let output_file = &fs.root_path.join("output.yaml");

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");

        let args = vec![
            &fs.root_path.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "--note",
            "This is a test note",
        ];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok());
            assert!(output_file.exists());

            // Verify that the note is included in the output file.
            let output_content = fs::read_to_string(output_file).unwrap();
            assert!(output_content.contains("This is a test note"));
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_existing_output_file() {
        let fs = TestFileSystem::new();
        
        let output_file = &fs.root_path.join("output.yaml");

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");

        // Create an existing output file with a "created" timestamp.
        fs::write(
            &output_file,
            r#"
created: "2020-01-01T00:00:00Z"
updated: "2020-01-02T00:00:00Z"
generated_comments: "Test comment"
directories:
src:
  main.rs: "fn main() {}"
"#,
        )
        .unwrap();

        let args = vec![
            &fs.root_path.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_final_println() {
        let fs = TestFileSystem::new();
        

        // Create a simple structure.
        // Create src directory via TestFileSystem helper
        // Directory created by fs.create_file
        fs.create_file("src/index.js", "console.log('Hello');");

        let args = vec![fs.root_path.to_str().unwrap()];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
}
