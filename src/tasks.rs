use crate::errors::SkeletorError;
use globset::GlobSet;
use indicatif::{ProgressBar, ProgressStyle};
use linked_hash_map::LinkedHashMap;
use log::{info, warn};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::Yaml;

/// A task to either create a directory or a file.
#[derive(Debug, PartialEq)]
pub enum Task {
    Dir(PathBuf),
    File(PathBuf, String),
}

/// Traverses the YAML structure and returns a list of tasks to create directories and files.
pub fn traverse_structure(base: &Path, yaml: &Yaml) -> Vec<Task> {
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
pub fn create_files_and_directories(
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
pub fn task_path(task: &Task) -> String {
    match task {
        Task::Dir(path) => format!("Dir: {:?}", path),
        Task::File(path, _) => format!("File: {:?}", path),
    }
}

/// Traverses the directory structure and returns a list of tasks to create a snapshot.
pub fn traverse_directory(
    base: &Path,
    include_contents: bool,
    ignore: Option<&GlobSet>,
) -> Result<(Yaml, Vec<String>), SkeletorError> {
    let mut mapping = LinkedHashMap::new();
    let mut binaries: Vec<String> = vec![];

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let new_relative = base.join(&file_name);

        if let Some(globset) = ignore {
            if globset.is_match(new_relative.to_string_lossy().as_ref()) {
                continue;
            }
        }

        let path = entry.path();
        if path.is_dir() {
            let (sub_yaml, mut sub_binaries) = traverse_directory(&path, include_contents, ignore)?;
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
                binaries.push(new_relative.to_string_lossy().into_owned());
            }
            mapping.insert(Yaml::String(file_name), Yaml::String(content));
        }
    }

    Ok((Yaml::Hash(mapping), binaries))
}

/// Computes statistics (number of files and directories) from a YAML structure.
pub fn compute_stats(yaml: &Yaml) -> (usize, usize) {
    let mut files = 0;
    let mut dirs = 0;
    if let Yaml::Hash(map) = yaml {
        for (_k, v) in map {
            match v {
                Yaml::Hash(_) => {
                    dirs += 1;
                    let (sub_files, sub_dirs) = compute_stats(v);
                    files += sub_files;
                    dirs += sub_dirs;
                }
                Yaml::String(_) => {
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
    use tempfile::tempdir;
    use yaml_rust::YamlLoader;

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
    fn test_compute_stats() {
        let yaml_str = r#"
        src:
          index.js: "console.log('Hello, world!');"
          components:
            Header.js: "// Header component"
        "#;
        let yaml = YamlLoader::load_from_str(yaml_str).unwrap()[0].clone();

        let (files, dirs) = compute_stats(&yaml);

        assert_eq!(files, 2);
        assert_eq!(dirs, 2); // One for "src" and one for "components"
    }
}
