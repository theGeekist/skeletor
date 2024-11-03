// src/main.rs

use clap::{Arg, ArgAction, Command};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use yaml_rust::YamlLoader;

#[derive(Debug, Error)]
enum SkeletorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] yaml_rust::ScanError),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, PartialEq)]
enum Task {
    Dir(PathBuf),
    File(PathBuf, String),
}

/// Parses command-line arguments and returns the matches.
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Skeletor")
    .version("1.0")
    .author("Your Name")
    .about("A super optimized Rust scaffolding tool")
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
    )
    .get_matches()
}

/// Reads the YAML configuration file and returns the parsed YAML node.
fn read_config(path: &Path) -> Result<yaml_rust::Yaml, SkeletorError> {
    let content = fs::read_to_string(path)?;
    let yaml_docs = YamlLoader::load_from_str(&content)?;
    
    // Access the "directories" key and return its content
    let directories = yaml_docs
    .get(0)
    .and_then(|doc| doc["directories"].as_hash())
    .ok_or_else(|| SkeletorError::Config("'directories' key is missing or invalid".into()))?;

Ok(yaml_rust::Yaml::Hash(directories.clone()))
}

/// Traverses the YAML structure and returns a list of tasks to create directories and files.
fn traverse_structure(base: &Path, yaml: &yaml_rust::Yaml) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((base.to_path_buf(), yaml));
    
    while let Some((current_path, node)) = queue.pop_front() {
        match node {
            yaml_rust::Yaml::Hash(map) => {
                for (key, value) in map {
                    if let yaml_rust::Yaml::String(key_str) = key {
                        let new_path = current_path.join(key_str);
                        match value {
                            yaml_rust::Yaml::Hash(_) => {
                                tasks.push(Task::Dir(new_path.clone()));
                                queue.push_back((new_path, value));
                            }
                            yaml_rust::Yaml::String(content) => {
                                tasks.push(Task::File(new_path, content.clone()));
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    tasks
}

/// Creates the files and directories as per the tasks, respecting the overwrite flag.
/// Continues on errors by design, logging any issues encountered.
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

/// Returns a string representation of the task for logging.
fn task_path(task: &Task) -> String {
    match task {
        Task::Dir(path) => format!("Dir: {:?}", path),
        Task::File(path, _) => format!("File: {:?}", path),
    }
}

fn main() -> Result<(), SkeletorError> {
    // Initialize the logger
    env_logger::init();
    
    let matches = parse_arguments();
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    
    fn setup_test_dir() -> PathBuf {
        let dir_number = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_dir = PathBuf::from(format!("test_skeletor_{}", dir_number));
    
        // Try removing the directory if it exists
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

    /// Tears down the test directory after the unit tests.
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
    fn test_parse_arguments_with_overwrite() {
        let args = vec![
            "skeletor",
            "--input",
            "structure.yaml",
            "--overwrite",
        ];
        let matches = Command::new("Skeletor")
            .version("1.0")
            .author("Your Name")
            .about("A super optimized Rust scaffolding tool")
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
            )
            .try_get_matches_from(&args)
            .unwrap();

        assert_eq!(
            matches.get_one::<String>("input").unwrap(),
            "structure.yaml"
        );
        assert_eq!(*matches.get_one::<bool>("overwrite").unwrap(), true);
    }

    #[test]
    fn test_parse_arguments_defaults() {
        let args = vec!["skeletor"];
        let matches = Command::new("Skeletor")
            .version("1.0")
            .author("Your Name")
            .about("A super optimized Rust scaffolding tool")
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
            )
            .try_get_matches_from(&args)
            .unwrap();

        assert_eq!(matches.get_one::<String>("input"), None);
        assert_eq!(*matches.get_one::<bool>("overwrite").unwrap_or(&false), false);
    }

    #[test]
    fn test_read_config_valid() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join(".skeletorrc");

    let yaml_content = r#"directories:
  src:
    index.js: "console.log('Hello, world!');"
    components:
      Header.js: "// Header component"
"#;

    fs::write(&config_file, yaml_content).unwrap();

    let result = read_config(&config_file);

    assert!(result.is_ok());

    // temp_dir will be deleted when it goes out of scope
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
    fn test_traverse_structure() {
        let structure = yaml_rust::YamlLoader::load_from_str(
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
}
