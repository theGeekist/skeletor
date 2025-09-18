use crate::errors::SkeletorError;
use globset::GlobSet;
use log::{info, warn};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of file and directory creation operations
#[derive(Debug, Clone)]
pub struct CreationResult {
    pub files_created: usize,
    pub dirs_created: usize,
    pub files_skipped: usize,
    pub skipped_files_list: Vec<String>,
    pub files_overwritten: usize,
    pub overwritten_files_list: Vec<String>,
}

impl Default for CreationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl CreationResult {
    pub fn new() -> Self {
        Self {
            files_created: 0,
            dirs_created: 0,
            files_skipped: 0,
            skipped_files_list: Vec::new(),
            files_overwritten: 0,
            overwritten_files_list: Vec::new(),
        }
    }
}

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
) -> Result<CreationResult, SkeletorError> {
    let mut result = CreationResult::new();

    for (i, task) in tasks.iter().enumerate() {
        match task {
            Task::Dir(path) => {
                if let Err(e) = fs::create_dir_all(path) {
                    warn!("Failed to create directory {:?}: {:?}", path, e);
                } else {
                    result.dirs_created += 1;
                    info!("Created directory: {:?}", path);
                }
            }
            Task::File(path, content) => {
                let file_exists = path.exists();
                
                if !overwrite && file_exists {
                    info!("Skipping file creation, already exists: {:?}", path);
                    result.files_skipped += 1;
                    result.skipped_files_list.push(path.display().to_string());
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
                        result.files_created += 1;
                        
                        if overwrite && file_exists {
                            result.files_overwritten += 1;
                            result.overwritten_files_list.push(path.display().to_string());
                            info!("Overwritten file: {:?}", path);
                        } else {
                            info!("Created file: {:?}", path);
                        }
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
        result.dirs_created, result.files_created
    );
    Ok(result)
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
                    // Use info logging for verbose ignore information
                    info!("Ignoring: {:?}", relative_str);
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
                    // Use warning log for file read errors instead of direct eprintln
                    warn!("Error reading file {:?}: {}", path, e);
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
    use crate::test_utils::helpers::*;

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
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

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
    fn test_traverse_directory() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        // Create a simple structure with a hidden file and a regular file.
        fs.create_file("src/index.js", "console.log('Hello');");
        // Hidden file should be included.
        fs.create_file("src/.hidden.txt", "secret");

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

    #[test]
    fn test_creation_result_new_and_default() {
        let result1 = CreationResult::new();
        let result2 = CreationResult::default();
        
        assert_eq!(result1.files_created, 0);
        assert_eq!(result1.dirs_created, 0);
        assert_eq!(result1.files_skipped, 0);
        assert_eq!(result1.files_overwritten, 0);
        assert!(result1.skipped_files_list.is_empty());
        assert!(result1.overwritten_files_list.is_empty());
        
        // Test that default and new produce equivalent results
        assert_eq!(result1.files_created, result2.files_created);
        assert_eq!(result1.dirs_created, result2.dirs_created);
    }

    #[test]
    fn test_creation_result_debug_and_clone() {
        let mut result = CreationResult::new();
        result.files_created = 5;
        result.files_skipped = 2;
        result.skipped_files_list.push("test.txt".to_string());
        
        // Test clone functionality
        let cloned = result.clone();
        assert_eq!(cloned.files_created, result.files_created);
        assert_eq!(cloned.skipped_files_list, result.skipped_files_list);
        
        // Test debug formatting
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("files_created"));
        assert!(debug_str.contains("files_skipped"));
    }

    #[test]
    fn test_create_files_and_directories_without_overwrite() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        // Create a file that already exists
        let existing_file = test_dir.join("existing.txt");
        fs.create_file("existing.txt", "original content");

        let tasks = vec![
            Task::File(existing_file.clone(), "new content".to_string()),
            Task::File(test_dir.join("new.txt"), "new file content".to_string()),
        ];

        let result = create_files_and_directories(&tasks, false).unwrap();
        
        // Should create 1 new file and skip 1 existing file
        assert_eq!(result.files_created, 1);
        assert_eq!(result.files_skipped, 1);
        assert_eq!(result.skipped_files_list.len(), 1);
        assert_eq!(result.files_overwritten, 0);
        assert!(result.overwritten_files_list.is_empty());
        
        // Verify original content wasn't overwritten
        let content = std::fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_create_files_and_directories_with_overwrite() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        // Create a file that already exists
        let existing_file = test_dir.join("existing.txt");
        fs.create_file("existing.txt", "original content");

        let tasks = vec![
            Task::File(existing_file.clone(), "overwritten content".to_string()),
            Task::File(test_dir.join("new.txt"), "new file content".to_string()),
        ];

        let result = create_files_and_directories(&tasks, true).unwrap();
        
        // Should create 2 files (1 new + 1 overwritten) and track overwrite
        assert_eq!(result.files_created, 2);
        assert_eq!(result.files_skipped, 0);
        assert_eq!(result.files_overwritten, 1);
        assert_eq!(result.overwritten_files_list.len(), 1);
        assert!(result.skipped_files_list.is_empty());
        
        // Verify content was overwritten
        let content = std::fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "overwritten content");
    }

    #[test]
    fn test_create_files_and_directories_with_directory_creation_failure() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        // Try to create a file in a deeply nested directory structure
        let nested_file = test_dir.join("deep/nested/structure/file.txt");
        let tasks = vec![
            Task::File(nested_file, "content".to_string()),
        ];

        // This should succeed because create_files_and_directories creates parent dirs
        let result = create_files_and_directories(&tasks, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.files_created, 1);
    }

    #[test]
    fn test_create_files_and_directories_progress_logging() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        // Create enough tasks to trigger progress logging (every 1000)
        let mut tasks = Vec::new();
        for i in 0..1005 {
            tasks.push(Task::File(
                test_dir.join(format!("file_{}.txt", i)),
                format!("content {}", i),
            ));
        }

        let result = create_files_and_directories(&tasks, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.files_created, 1005);
    }

    #[test]
    fn test_traverse_directory_with_include_contents() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        fs.create_file("text.txt", "Hello, world!");
        fs.create_binary_file("binary.bin", &[0xFF, 0xFE, 0xFD, 0xFC]);

        let (yaml_structure, binaries) = traverse_directory(test_dir, true, None, false).unwrap();

        // With include_contents=true, should detect binary files
        assert!(!binaries.is_empty());
        
        if let Value::Mapping(map) = yaml_structure {
            // Text file should be included in YAML
            assert!(map.contains_key(Value::String("text.txt".into())));
            // Binary file should NOT be in YAML content (tracked in binaries list)
            assert!(!map.contains_key(Value::String("binary.bin".into())));
        } else {
            panic!("Expected a YAML mapping");
        }
    }

    #[test]
    fn test_traverse_directory_with_verbose_logging() {
        let fs = TestFileSystem::new();
        let test_dir = &fs.root_path;

        fs.create_file("normal.txt", "content");

        // Test verbose mode (should log more information)
        let result = traverse_directory(test_dir, false, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_traverse_structure_with_non_mapping_values() {
        let structure: Value = serde_yaml::from_str(
            r#"
            src:
              file.txt: "content"
              number: 42
              boolean: true
              array: [1, 2, 3]
            "#,
        ).unwrap();

        let tasks = traverse_structure(Path::new("."), &structure);

        // Only string values should create file tasks
        let file_tasks: Vec<_> = tasks.iter().filter(|t| matches!(t, Task::File(_, _))).collect();
        assert_eq!(file_tasks.len(), 1);
        
        let dir_tasks: Vec<_> = tasks.iter().filter(|t| matches!(t, Task::Dir(_))).collect();
        assert_eq!(dir_tasks.len(), 1); // Just the "src" directory
    }

    #[test]
    fn test_traverse_structure_empty_input() {
        let empty_structure = Value::Mapping(serde_yaml::Mapping::new());
        let tasks = traverse_structure(Path::new("."), &empty_structure);
        assert!(tasks.is_empty());
    }

    #[test] 
    fn test_compute_stats_empty_structure() {
        let empty_yaml = Value::Mapping(serde_yaml::Mapping::new());
        let (files, dirs) = compute_stats(&empty_yaml);
        assert_eq!(files, 0);
        assert_eq!(dirs, 0);
    }

    #[test]
    fn test_compute_stats_non_string_values() {
        let yaml_str = r#"
        root:
          file.txt: "content"
          number: 42
          boolean: true
          nested:
            another.txt: "more content"
        "#;
        let yaml: Value = serde_yaml::from_str(yaml_str).unwrap();
        let (files, dirs) = compute_stats(&yaml);
        
        // Should only count string values as files
        assert_eq!(files, 2);  // file.txt and another.txt
        assert_eq!(dirs, 2);   // root and nested
    }
}
