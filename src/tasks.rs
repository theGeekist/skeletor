use crate::errors::SkeletorError;
use globset::GlobSet;
use log::{info, warn};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// A task to either create a directory or a file.
#[derive(Debug, PartialEq)]
pub enum Task {
    Dir(PathBuf),
    File(PathBuf, String),
}

/// Traverses the YAML structure and returns a list of tasks to create directories and files.
pub fn traverse_structure(base: &Path, yaml: &Value) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut queue = Vec::new();
    queue.push((base.to_path_buf(), yaml));

    while let Some((current_path, node)) = queue.pop() {
        if let Some(map) = node.as_mapping() {
            for (key, value) in map {
                if let Some(key_str) = key.as_str() {
                    let new_path = current_path.join(key_str);
                    match value {
                        Value::Mapping(_) => {
                            tasks.push(Task::Dir(new_path.clone()));
                            queue.push((new_path, value));
                        }
                        Value::String(content) => {
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
pub fn create_files_and_directories(
    tasks: &[Task],
    overwrite: bool,
) -> Result<(usize, usize), SkeletorError> {
    let mut files_created = 0;
    let mut dirs_created = 0;

    for (i, task) in tasks.iter().enumerate() {
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

        // **Log Progress Every 100 Files to Avoid IO Overhead**
        if i % 1000 == 0 && i > 0 {
            info!("Processed {} out of {} tasks...", i, tasks.len());
        }
    }

    info!(
        "Task Complete: {} directories and {} files created.",
        dirs_created, files_created
    );
    Ok((files_created, dirs_created))
}
/// Returns a string representation of a task.
pub fn task_path(task: &Task) -> String {
    match task {
        Task::Dir(path) => format!("Dir: {:?}", path),
        Task::File(path, _) => format!("File: {:?}", path),
    }
}

pub fn traverse_directory(
    base: &Path,
    include_contents: bool,
    ignore: Option<&GlobSet>,
    verbose: bool,
) -> Result<(Value, Vec<String>), SkeletorError> {
    let mut mapping = serde_yaml::Mapping::new();
    let mut binaries: Vec<String> = vec![];

    for entry in fs::read_dir(base).map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::NotFound => SkeletorError::directory_not_found(base.to_path_buf()),
            _ => SkeletorError::from_io_with_context(e, base.to_path_buf())
        }
    })? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_string = file_name.to_string_lossy().into_owned();
        let new_relative = base.join(&file_name_string);

        // ✅ Normalize path to relative string
        let mut relative_str = new_relative
            .strip_prefix(base)
            .unwrap_or(&new_relative)
            .to_string_lossy()
            .replace("\\", "/");

        // ✅ If it's a directory, append `/` to match `.gitignore`
        if new_relative.is_dir() {
            relative_str.push('/');
        }

        if let Some(globset) = ignore {
            if globset.is_match(&relative_str) {
                if verbose {
                    println!("Ignoring: {:?}", relative_str);
                }
                continue;
            }
        }

        let path = entry.path();
        if path.is_dir() {
            let (sub_yaml, mut sub_binaries) = traverse_directory(&path, include_contents, ignore, verbose)?;
            mapping.insert(Value::String(file_name_string), sub_yaml);
            binaries.append(&mut sub_binaries);
        } else if path.is_file() && include_contents {
            match fs::read(&path) {
                Ok(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes.clone()) {
                        // println!("Storing file: {:?}", path);
                        mapping.insert(Value::String(file_name_string), Value::String(text));
                    } else {
                        // println!("Binary file detected: {:?}", path);
                        binaries.push(new_relative.to_string_lossy().into_owned());
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file {:?}: {}", path, e);
                }
            }
        }
    }

    Ok((Value::Mapping(mapping), binaries))
}

/// Computes statistics (number of files and directories) from a YAML structure.
pub fn compute_stats(yaml: &Value) -> (usize, usize) {
    let mut files = 0;
    let mut dirs = 0;

    if let Some(map) = yaml.as_mapping() {
        for (_, v) in map {
            match v {
                Value::Mapping(_) => {
                    dirs += 1;
                    let (sub_files, sub_dirs) = compute_stats(v);
                    files += sub_files;
                    dirs += sub_dirs;
                }
                Value::String(_) => {
                    files += 1;
                }
                _ => {}
            }
        }
    }

    (files, dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_traverse_structure() {
        let structure: Value = serde_yaml::from_str(
            r#"
            src:
              index.js: "console.log('Hello, world!');"
              components:
                Header.js: "// Header component"
            "#,
        )
        .expect("Failed to parse YAML");

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
            assert!(map.contains_key(Value::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
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
        let yaml: Value = serde_yaml::from_str(yaml_str).expect("Failed to parse YAML");

        let (files, dirs) = compute_stats(&yaml);

        assert_eq!(files, 2);
        assert_eq!(dirs, 2); // One for "src" and one for "components"
    }
}
