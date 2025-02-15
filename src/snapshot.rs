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

/// Runs the snapshot subcommand: Generates a structured snapshot and writes it to disk.
/// Runs the snapshot subcommand: Generates a structured snapshot and writes it to disk.
pub fn run_snapshot(matches: &ArgMatches) -> Result<(), SkeletorError> {
    let source_path = PathBuf::from(matches.get_one::<String>("source").unwrap());
    let output_path = default_file_path(matches.get_one::<String>("output"));
    let include_contents = matches.get_flag("include_contents"); // No forced `true`
    let dry_run = matches.get_flag("dry_run");
    let user_note = matches.get_one::<String>("note").map(|s| s.to_string());

    info!("Taking snapshot of folder: {:?}", source_path);
    let start_time = Instant::now();

    // ✅ Process ignore patterns correctly
    let ignore_patterns = collect_ignore_patterns(matches)?;
    println!("Loaded ignore patterns: {:?}", ignore_patterns); // Debugging

    let globset = build_globset(&ignore_patterns)?;

    // ✅ Take the snapshot (Directory Traversal)
    let (dir_snapshot, binary_files) =
        traverse_directory(&source_path, include_contents, globset.as_ref())?;
    let (files_count, dirs_count) = compute_stats(&dir_snapshot);

    // ✅ Build snapshot metadata
    let snapshot = build_snapshot(
        &output_path,
        user_note,
        dir_snapshot,
        binary_files,
        files_count,
        dirs_count,
    )?;

    // ✅ Output the snapshot
    let duration = start_time.elapsed();
    write_snapshot(snapshot, &output_path, dry_run)?;

    println!("\nSnapshot generated in {:?}", duration);
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
                let content = fs::read_to_string(candidate)?;
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

fn build_globset(ignore_patterns: &[String]) -> Result<Option<GlobSet>, SkeletorError> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pat in ignore_patterns {
        let normalized_pattern = pat.trim().to_string();
        match Glob::new(&normalized_pattern) {
            Ok(glob) => {
                println!("Added ignore pattern: {}", normalized_pattern);
                builder.add(glob);
            }
            Err(e) => {
                eprintln!("Invalid glob pattern: {} - {}", normalized_pattern, e);
            }
        }
    }

    builder
        .build()
        .map(Some)
        .map_err(|e| SkeletorError::Config(format!("Failed to build GlobSet: {}", e)))
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
fn write_snapshot(snapshot: Value, output_path: &Path, dry_run: bool) -> Result<(), SkeletorError> {
    let out_str =
        serde_yaml::to_string(&snapshot).map_err(|e| SkeletorError::Config(e.to_string()))?;

    if dry_run {
        println!("Dry run enabled. The following snapshot would be generated:");
        println!("{}", out_str);
    } else {
        fs::write(output_path, out_str)?;
        println!("Snapshot written to {:?}", output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::*;
    use clap::{Arg, ArgAction, Command, ArgMatches};
    use tempfile::tempdir;

    /// Helper function to parse CLI args for testing.
    fn get_test_matches(subcommand: &str, args: &[&str]) -> Option<ArgMatches> {
        let command = Command::new("skeletor")
            .subcommand(
                Command::new("snapshot")
                    .about("Create a snapshot of a directory")
                    .arg(Arg::new("source").value_name("FOLDER").required(true))
                    .arg(Arg::new("output").short('o').long("output").value_name("FILE"))
                    .arg(Arg::new("include_contents").long("include-contents").action(ArgAction::SetTrue))
                    .arg(Arg::new("ignore").short('I').long("ignore").value_name("PATTERN_OR_FILE").action(ArgAction::Append))
                    .arg(Arg::new("dry_run").short('d').long("dry-run").action(ArgAction::SetTrue))
                    .arg(Arg::new("note").short('n').long("note").value_name("NOTE")),
            );
    
        // 🛠 Fix: Ensure `"skeletor"` comes first, then `"snapshot"`, then `args`
        let args_iter = std::iter::once("skeletor").chain(std::iter::once(subcommand)).chain(args.iter().copied());
    
        match command.try_get_matches_from(args_iter) {
            Ok(matches) => matches.subcommand_matches(subcommand).cloned(),
            Err(e) => {
                println!("❌ Failed to parse arguments: {:?}", e);
                None
            }
        }
    }
    
    

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

        let (yaml_structure, binaries) = traverse_directory(&test_dir, false, None).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Value::String("src".to_string())));
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

        let (yaml_structure, binaries) = traverse_directory(&test_dir, true, None).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Value::String("src".to_string())));
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

        let (yaml_structure, binaries) = traverse_directory(&test_dir, false, None).unwrap();

        if let Value::Mapping(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Value::String("src".to_string())));
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

        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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

        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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
        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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
        fs::write(src.join("binary.bin"), &[0, 159, 146, 150]).unwrap();

        let args = vec![test_dir.to_str().unwrap()];
        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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
        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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
        if let Some(sub_m) = get_test_matches("snapshot", &args) {
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
        if let Some(sub_m) = get_test_matches("snapshot", &args) {
            let result = run_snapshot(&sub_m);
            assert!(result.is_ok(), "run_snapshot failed: {:?}", result);
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
}
