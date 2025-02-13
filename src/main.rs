// src/main.rs

use clap::{Arg, ArgAction, Command};
use indicatif::{ProgressBar, ProgressStyle};
use linked_hash_map::LinkedHashMap;
use log::{error, info, warn};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

/// For timestamp formatting, we use chrono.
use chrono::Utc;

#[derive(Debug, Error)]
enum SkeletorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] yaml_rust::ScanError),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// A task to either create a directory or a file.
#[derive(Debug, PartialEq)]
enum Task {
    Dir(PathBuf),
    File(PathBuf, String),
}

/// Build the CLI interface with three subcommands: `apply`, `snapshot` and `info`
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Skeletor")
        .version("2.1.0")
        .author("Jason Joseph Nathan")
        .about("A super optimised Rust scaffolding tool with snapshot annotations")
        .subcommand_required(true)
        .subcommand(
            Command::new("apply")
                .about("Applies a YAML configuration to generate files and directories")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the input YAML configuration file (defaults to .skeletorrc)"),
                )
                .arg(
                    Arg::new("overwrite")
                        .short('o')
                        .long("overwrite")
                        .help("Overwrite existing files if specified")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Generates a .skeletorrc snapshot from an existing folder")
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
                        .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                )
                .arg(
                    Arg::new("include_contents")
                        .long("include-contents")
                        .help("Include file contents in the snapshot (for text files; binary files will be empty)")
                        .action(ArgAction::SetTrue),
                )
                // Although a flag existed before, we now always include hidden files/folders.
        )
        .subcommand(
            Command::new("info")
                .about("Prints annotation information from a .skeletorrc file")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file (defaults to .skeletorrc)"),
                ),
        )
        .get_matches()
}

/// Reads the YAML configuration file and extracts the "directories" key.
/// (Any extra keys are ignored by the apply functionality.)
fn read_config(path: &Path) -> Result<Yaml, SkeletorError> {
    let content = fs::read_to_string(path)?;
    let yaml_docs = YamlLoader::load_from_str(&content)?;

    let directories = yaml_docs
        .first()
        .and_then(|doc| doc["directories"].as_hash())
        .ok_or_else(|| SkeletorError::Config("'directories' key is missing or invalid".into()))?;

    Ok(Yaml::Hash(directories.clone()))
}

/// Traverses the YAML structure and returns a list of tasks to create directories and files.
fn traverse_structure(base: &Path, yaml: &Yaml) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((base.to_path_buf(), yaml));

    while let Some((current_path, node)) = queue.pop_front() {
        if let Yaml::Hash(map) = node {
            for (key, value) in map {
                if let Yaml::String(key_str) = key {
                    let new_path = current_path.join(key_str);
                    match value {
                        Yaml::Hash(_) => {
                            tasks.push(Task::Dir(new_path.clone()));
                            queue.push_back((new_path, value));
                        }
                        Yaml::String(content) => {
                            tasks.push(Task::File(new_path, content.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    tasks
}

/// Creates files and directories as specified by tasks; logs progress and respects the overwrite flag.
fn create_files_and_directories(
    tasks: &[Task],
    overwrite: bool,
) -> Result<(usize, usize), SkeletorError> {
    let total_tasks = tasks.len() as u64;
    let pb = ProgressBar::new(total_tasks);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) - {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut files_created = 0;
    let mut dirs_created = 0;

    for task in tasks {
        match task {
            Task::Dir(path) => {
                if let Err(e) = fs::create_dir_all(path) {
                    warn!("Failed to create directory {:?}: {:?}", path, e);
                } else {
                    dirs_created += 1;
                    info!("Created directory: {:?}", path);
                }
            }
            Task::File(path, content) => {
                if !overwrite && path.exists() {
                    info!("Skipping file creation, already exists: {:?}", path);
                } else {
                    if let Some(parent) = path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            warn!(
                                "Failed to create parent directory for file {:?}: {:?}",
                                path, e
                            );
                            continue;
                        }
                    }
                    if let Err(e) = fs::write(path, content) {
                        warn!("Failed to write file {:?}: {:?}", path, e);
                    } else {
                        files_created += 1;
                        info!("Created file: {:?}", path);
                    }
                }
            }
        }
        pb.inc(1);
        pb.set_message(format!("Processing: {}", task_path(task)));
    }

    pb.finish_with_message("Done");
    Ok((files_created, dirs_created))
}

/// Returns a string representation of a task.
fn task_path(task: &Task) -> String {
    match task {
        Task::Dir(path) => format!("Dir: {:?}", path),
        Task::File(path, _) => format!("File: {:?}", path),
    }
}

/// Recursively snapshots a directory into a YAML structure.  
/// Returns a tuple of:
/// - The YAML representation (a Hash mapping names to file contents or subdirectories)
/// - A vector of relative file paths that were detected as binary.
fn snapshot_directory(
    base: &Path,
    include_contents: bool,
    // Hidden files are always included to match original behaviour.
    _include_hidden: bool,
) -> Result<(Yaml, Vec<String>), SkeletorError> {
    let mut mapping = LinkedHashMap::new(); // Use LinkedHashMap instead of BTreeMap
    let mut binaries: Vec<String> = vec![];

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        if path.is_dir() {
            let (sub_yaml, mut sub_binaries) = snapshot_directory(&path, include_contents, true)?;
            mapping.insert(Yaml::String(file_name), sub_yaml);
            binaries.append(&mut sub_binaries);
        } else if path.is_file() {
            let mut is_binary = false;
            let content = if include_contents {
                let bytes = fs::read(&path)?;
                match String::from_utf8(bytes) {
                    Ok(text) => text,
                    Err(_) => {
                        is_binary = true;
                        String::new()
                    }
                }
            } else {
                String::new()
            };

            if is_binary {
                // Record the relative file path; for simplicity, just the file name.
                binaries.push(file_name.clone());
            }
            mapping.insert(Yaml::String(file_name), Yaml::String(content));
        }
    }

    Ok((Yaml::Hash(mapping), binaries))
}

/// Runs the snapshot subcommand. Generates a snapshot with extra annotations.
fn run_snapshot(matches: &clap::ArgMatches) -> Result<(), SkeletorError> {
    let source_path = PathBuf::from(matches.get_one::<String>("source").unwrap());
    let output_path = matches.get_one::<String>("output").map(PathBuf::from);
    let include_contents = *matches
        .get_one::<bool>("include_contents")
        .unwrap_or(&false);

    info!("Taking snapshot of folder: {:?}", source_path);
    // Always include hidden files to match original behaviour.
    let (dir_snapshot, binary_files) = snapshot_directory(&source_path, include_contents, true)?;

    let current_date = Utc::now().to_rfc3339();
    let mut comments = format!("Snapshot generated from folder: {:?}", source_path);
    if binary_files.is_empty() {
        comments.push_str("\nNo binary files detected.");
    } else {
        comments.push_str(&format!(
            "\nBinary files detected (contents omitted): {:?}",
            binary_files
        ));
    }

    // Build the top-level YAML mapping without changing the existing structure.
    let mut top_map = LinkedHashMap::new(); // Use LinkedHashMap instead of BTreeMap
    top_map.insert(Yaml::String("dates".into()), Yaml::String(current_date));
    top_map.insert(Yaml::String("comments".into()), Yaml::String(comments));
    top_map.insert(Yaml::String("directories".into()), dir_snapshot);
    let snapshot_yaml = Yaml::Hash(top_map);

    // Emit the YAML.
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&snapshot_yaml).unwrap();
    }

    if let Some(out_file) = output_path {
        fs::write(&out_file, out_str.clone())?;
        println!("Snapshot written to {:?}", out_file);
    } else {
        println!("{}", out_str);
    }

    Ok(())
}

/// Runs the apply subcommand: reads the YAML config and creates files/directories.
fn run_apply(matches: &clap::ArgMatches) -> Result<(), SkeletorError> {
    let input_path = matches
        .get_one::<String>("input")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".skeletorrc"));

    let overwrite = *matches.get_one::<bool>("overwrite").unwrap_or(&false);

    info!("Reading input file: {:?}", input_path);
    info!("Overwrite flag: {:?}", overwrite);

    let config = read_config(&input_path)?;

    if config.is_null() {
        return Err(SkeletorError::Config(
            "'directories' key is required in the YAML file".into(),
        ));
    }

    let start_time = Instant::now();

    let tasks = traverse_structure(Path::new("."), &config);

    create_files_and_directories(&tasks, overwrite)?;

    let duration = start_time.elapsed();
    println!(
        "\nSuccessfully generated files and directories in {:?}.",
        duration
    );

    Ok(())
}

/// Runs the info subcommand: prints annotation information from a .skeletorrc file.
fn run_info(matches: &clap::ArgMatches) -> Result<(), SkeletorError> {
    let input_path = matches
        .get_one::<String>("input")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".skeletorrc"));

    let content = fs::read_to_string(&input_path)?;
    let yaml_docs = YamlLoader::load_from_str(&content)?;
    let doc = &yaml_docs[0];

    println!("Information from {:?}:", input_path);
    if let Some(date) = doc["dates"].as_str() {
        println!("  Snapshot date: {}", date);
    } else {
        println!("  No date information available.");
    }
    if let Some(comments) = doc["comments"].as_str() {
        println!("  Comments: {}", comments);
    } else {
        println!("  No comments available.");
    }
    Ok(())
}

fn main() -> Result<(), SkeletorError> {
    env_logger::init();

    let matches = parse_arguments();

    match matches.subcommand() {
        Some(("apply", sub_m)) => run_apply(sub_m)?,
        Some(("snapshot", sub_m)) => run_snapshot(sub_m)?,
        Some(("info", sub_m)) => run_info(sub_m)?,
        _ => unreachable!("A subcommand is required"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn setup_test_dir() -> PathBuf {
        let dir_number = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_dir = PathBuf::from(format!("test_skeletor_{}", dir_number));

        if test_dir.exists() {
            for _ in 0..3 {
                if let Err(e) = fs::remove_dir_all(&test_dir) {
                    println!("Failed to remove test directory, retrying: {:?}", e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                } else {
                    break;
                }
            }
        }

        fs::create_dir(&test_dir).expect("Failed to create test directory");
        test_dir
    }

    fn teardown_test_dir(test_dir: &PathBuf) {
        if test_dir.exists() {
            for _ in 0..3 {
                if let Err(e) = fs::remove_dir_all(test_dir) {
                    println!("Failed to clean up test directory, retrying: {:?}", e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                } else {
                    break;
                }
            }
        }
    }

    #[test]
    fn test_parse_arguments_with_overwrite_apply() {
        let args = vec![
            "skeletor",
            "apply",
            "--input",
            "structure.yaml",
            "--overwrite",
        ];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("apply")
                    .arg(
                        Arg::new("input")
                            .short('i')
                            .long("input")
                            .value_name("FILE")
                            .help("Specify the input YAML configuration file"),
                    )
                    .arg(
                        Arg::new("overwrite")
                            .short('o')
                            .long("overwrite")
                            .help("Overwrite existing files if specified")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            assert_eq!(sub_m.get_one::<String>("input").unwrap(), "structure.yaml");
            assert_eq!(*sub_m.get_one::<bool>("overwrite").unwrap(), true);
        } else {
            panic!("Apply subcommand not found");
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

        let (yaml_structure, binaries) = snapshot_directory(&test_dir, false, true).unwrap();

        if let Yaml::Hash(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_traverse_structure() {
        let structure = YamlLoader::load_from_str(
            r#"
            src:
              index.js: "console.log('Hello, world!');"
              components:
                Header.js: "// Header component"
            "#,
        )
        .unwrap()[0]
            .clone();

        let tasks = traverse_structure(Path::new("."), &structure);

        let expected_tasks = vec![
            Task::Dir(Path::new("./src").to_path_buf()),
            Task::File(
                Path::new("./src/index.js").to_path_buf(),
                "console.log('Hello, world!');".to_string(),
            ),
            Task::Dir(Path::new("./src/components").to_path_buf()),
            Task::File(
                Path::new("./src/components/Header.js").to_path_buf(),
                "// Header component".to_string(),
            ),
        ];

        assert_eq!(tasks, expected_tasks);
    }

    #[test]
    fn test_create_files_and_directories() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        let tasks = vec![
            Task::Dir(test_dir.join("src")),
            Task::File(
                test_dir.join("src/index.js"),
                "console.log('Hello, world!');".to_string(),
            ),
            Task::Dir(test_dir.join("src/components")),
            Task::File(
                test_dir.join("src/components/Header.js"),
                "// Header component".to_string(),
            ),
        ];

        let result = create_files_and_directories(&tasks, true);
        assert!(result.is_ok());

        assert!(test_dir.join("src/index.js").exists());
        assert!(test_dir.join("src/components/Header.js").exists());
    }

    #[test]
    fn test_task_path() {
        let dir_task = Task::Dir(PathBuf::from("src"));
        let file_task = Task::File(
            PathBuf::from("src/index.js"),
            "console.log('Hello, world!');".to_string(),
        );

        assert_eq!(task_path(&dir_task), "Dir: \"src\"");
        assert_eq!(task_path(&file_task), "File: \"src/index.js\"");
    }

    #[test]
    fn test_read_config_invalid() {
        let test_dir = setup_test_dir();
        let config_file = test_dir.join("invalid.yaml");

        let invalid_yaml_content = "invalid_yaml: data\n\tbad_indent: - missing_value";
        fs::write(&config_file, invalid_yaml_content).unwrap();

        let result = read_config(&config_file);

        assert!(result.is_err());

        teardown_test_dir(&test_dir);
    }

    #[test]
    fn read_config_valid() {
        let yaml_str = r#"
        directories:
          src:
            index.js: "console.log('Hello, world!');"
            components:
              Header.js: "// Header component"
        "#;
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.yaml");
        fs::write(&test_file, yaml_str).unwrap();

        let config = read_config(&test_file).unwrap();
        if let Yaml::Hash(map) = config {
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
    }
}
