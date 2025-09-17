use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::tasks::{compute_stats, traverse_directory};
use chrono::Utc;
use clap::ArgMatches;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::info;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use termcolor::{StandardStream, ColorChoice, Color, ColorSpec, WriteColor};
use std::io::Write;

/// Helper function to print colored timing information with standardized formatting
fn print_colored_duration(prefix: &str, duration: std::time::Duration) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    print!("{}", prefix);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
    let _ = write!(stdout, "{:.2}ms", duration.as_micros() as f64 / 1000.0);
    let _ = stdout.reset();
    println!();
}

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
        println!("Using {} ignore pattern(s)", ignore_patterns.len());
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
        dir_snapshot,
        binary_files,
        files_count,
        dirs_count,
    )?;

    let duration = start_time.elapsed();
    write_snapshot(snapshot, &config.output_path, config.dry_run, verbose_info)?;
    print_colored_duration("Snapshot generated in ", duration);
    
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
                let content = fs::read_to_string(candidate)
                    .map_err(|e| SkeletorError::from_io_with_context(e, candidate.to_path_buf()))?;
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

fn build_globset(ignore_patterns: &[String], verbose: bool) -> Result<Option<GlobSet>, SkeletorError> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pat in ignore_patterns {
        let normalized_pattern = pat.trim().to_string();
        match Glob::new(&normalized_pattern) {
            Ok(glob) => {
                if verbose {
                    println!("Added ignore pattern: {}", normalized_pattern);
                }
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

/// Writes the snapshot to disk or prints it for dry-run mode.
fn write_snapshot(snapshot: Value, output_path: &Path, dry_run: bool, verbose_info: Vec<String>) -> Result<(), SkeletorError> {
    let out_str =
        serde_yaml::to_string(&snapshot).map_err(|e| SkeletorError::Config(e.to_string()))?;

    if dry_run {
        // For snapshot, always show the full YAML content as it's typically readable
        // The verbose flag controls additional metadata display
        println!("Dry run enabled. The following snapshot would be generated:");
        println!("{}", out_str);
        
        // Display verbose information at the end only if verbose
        if !verbose_info.is_empty() {
            println!("Verbose Information:");
            for info in verbose_info {
                println!("{}", info);
            }
        }
    } else {
        fs::write(output_path, out_str)
            .map_err(|e| SkeletorError::from_io_with_context(e, output_path.to_path_buf()))?;
        println!("Snapshot written to {:?}", output_path);
        
        // Display verbose information at the end for non-dry-run too
        if !verbose_info.is_empty() {
            for info in verbose_info {
                println!("{}", info);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_directory_without_contents() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(test_dir, false, None, false).unwrap();

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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(test_dir, true, None, false).unwrap();

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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(test_dir, false, None, false).unwrap();

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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec![test_dir.to_str().unwrap(), "--dry-run"];

        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_output() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let output_file = temp_dir.path().join("output.yaml");

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec![
            test_dir.to_str().unwrap(),
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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        fs::write(src.join("ignore.txt"), "ignore me").unwrap();

        let ignore_file = temp_dir.path().join("ignore_patterns.txt");
        fs::write(&ignore_file, "ignore.txt").unwrap();

        let args = vec![
            test_dir.to_str().unwrap(),
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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a binary file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        fs::write(src.join("binary.bin"), [0, 159, 146, 150]).unwrap();

        let args = vec![test_dir.to_str().unwrap()];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
    #[test]
    fn test_run_snapshot_with_notes() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let output_file = temp_dir.path().join("output.yaml");

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec![
            test_dir.to_str().unwrap(),
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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let output_file = temp_dir.path().join("output.yaml");

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

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
            test_dir.to_str().unwrap(),
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
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec![test_dir.to_str().unwrap()];
        if let Some(sub_m) = crate::test_utils::helpers::create_snapshot_matches(args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
}
