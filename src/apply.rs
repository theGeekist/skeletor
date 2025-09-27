use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::output::{DefaultReporter, Reporter, SimpleApplyResult};
use crate::tasks::{create_files_and_directories, traverse_structure, Task};
use clap::ArgMatches;
use log::info;
use serde_yaml::Value;
use std::time::Instant;

/// Extract binary files list from YAML if present
fn extract_binary_files_from_yaml(yaml_config: &Value) -> Vec<String> {
    if let Some(binary_files) = yaml_config.get("binary_files") {
        if let Some(array) = binary_files.as_sequence() {
            return array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }
    }
    Vec::new()
}

/// Extract ignore patterns from YAML if present
fn extract_ignore_patterns_from_yaml(yaml_config: &Value) -> Vec<String> {
    if let Some(ignore_patterns) = yaml_config.get("ignore_patterns") {
        if let Some(array) = ignore_patterns.as_sequence() {
            return array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }
    }
    Vec::new()
}

/// Handles dry-run output display using the Reporter system for consistent formatting
fn display_dry_run_output(tasks: &[Task], verbose: bool, binary_files: &[String], ignore_patterns: &[String]) {
    let reporter = DefaultReporter::new();
            reporter.dry_run_preview_comprehensive(tasks, verbose, binary_files, ignore_patterns, "applied");
}

/// Parses CLI arguments and extracts apply-specific configuration
struct ApplyConfig {
    pub input_path: std::path::PathBuf,
    pub output_dir: std::path::PathBuf,
    pub overwrite: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ApplyConfig {
    fn from_matches(matches: &ArgMatches) -> Self {
        let output_dir = matches
            .get_one::<String>("output")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        
        Self {
            input_path: default_file_path(matches.get_one::<String>("config")),
            output_dir,
            overwrite: *matches.get_one::<bool>("overwrite").unwrap_or(&false),
            dry_run: matches.get_flag("dry_run"),
            verbose: matches.get_flag("verbose"),
        }
    }
}

/// Runs the apply subcommand: reads the YAML config and creates files/directories.
/// In dry-run mode, the tasks are printed without performing any filesystem changes.
pub fn run_apply(matches: &ArgMatches) -> Result<(), SkeletorError> {
    let config = ApplyConfig::from_matches(matches);

    info!("Reading input file: {:?}", config.input_path);
    info!("Overwrite flag: {:?}", config.overwrite);

    // Read the full YAML document to access binary_files and ignore_patterns
    let full_yaml_doc: Value = crate::utils::read_yaml_file(&config.input_path)?;
    
    // Extract directories section for processing
    let yaml_config = read_config(&config.input_path)?;

    if yaml_config.is_null() {
        return Err(SkeletorError::Config(
            "'directories' key is required in the YAML file".into(),
        ));
    }

    let start_time = Instant::now();
    let tasks = traverse_structure(&config.output_dir, &yaml_config);
    
    // Extract binary files and ignore patterns from the full YAML document
    let binary_files = extract_binary_files_from_yaml(&full_yaml_doc);
    let ignore_patterns = extract_ignore_patterns_from_yaml(&full_yaml_doc);
    
    info!("Extracted {} binary files: {:?}", binary_files.len(), binary_files);
    info!("Extracted {} ignore patterns: {:?}", ignore_patterns.len(), ignore_patterns);

    if config.dry_run {
        display_dry_run_output(&tasks, config.verbose, &binary_files, &ignore_patterns);
    } else {
        let reporter = DefaultReporter::new();
        
        if config.verbose {
            reporter.verbose_operation_preview(&tasks);
        } else {
            reporter.operation_start("apply", &format!("Creating {} tasks", tasks.len()));
        }
        
        let creation_result = create_files_and_directories(&tasks, config.overwrite)?;
        let duration = start_time.elapsed();
        
        let apply_result = SimpleApplyResult::with_skipped_and_overwritten(
            creation_result.files_created,
            creation_result.dirs_created,
            duration,
            tasks.len(),
            creation_result.files_skipped,
            creation_result.skipped_files_list,
            creation_result.files_overwritten,
            creation_result.overwritten_files_list,
        );
        reporter.apply_complete(&apply_result, config.verbose);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test_utils::helpers::*;

    #[test]
    fn test_parse_arguments_with_overwrite_apply() {
        let args = vec![
            "structure.yaml",
            "--overwrite",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_eq!(sub_m.get_one::<String>("config").unwrap(), "structure.yaml");
            assert!(*sub_m.get_one::<bool>("overwrite").unwrap());
            assert!(!(*sub_m.get_one::<bool>("dry_run").unwrap_or(&false)));
        } else {
            panic!("Apply subcommand not found");
        }
    }

    #[test]
    fn test_apply_with_missing_config_file() {
        let fs = TestFileSystem::new();
        let non_existent_file = fs.path("missing.yml");
        
        let args = vec![non_existent_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_with_dry_run() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "--dry-run",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_with_invalid_yaml() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_invalid_config("invalid.yml");
        
        let args = vec![config_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_without_directories_key() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_config_without_directories("no_dirs.yml");
        
        let args = vec![config_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_extract_binary_files_from_yaml() {
        use serde_yaml::Value;
        
        // Test with binary_files present
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            Value::String("binary_files".to_string()),
            Value::Sequence(vec![
                Value::String("image.png".to_string()),
                Value::String("binary.exe".to_string()),
            ])
        );
        let yaml = Value::Mapping(mapping);
        
        let result = super::extract_binary_files_from_yaml(&yaml);
        assert_eq!(result, vec!["image.png", "binary.exe"]);
        
        // Test with no binary_files key
        let empty_yaml = Value::Mapping(serde_yaml::Mapping::new());
        let result = super::extract_binary_files_from_yaml(&empty_yaml);
        assert!(result.is_empty());
        
        // Test with non-sequence binary_files
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            Value::String("binary_files".to_string()),
            Value::String("not_a_sequence".to_string())
        );
        let yaml = Value::Mapping(mapping);
        
        let result = super::extract_binary_files_from_yaml(&yaml);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_ignore_patterns_from_yaml() {
        use serde_yaml::Value;
        
        // Test with ignore_patterns present
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            Value::String("ignore_patterns".to_string()),
            Value::Sequence(vec![
                Value::String("*.tmp".to_string()),
                Value::String("target/".to_string()),
            ])
        );
        let yaml = Value::Mapping(mapping);
        
        let result = super::extract_ignore_patterns_from_yaml(&yaml);
        assert_eq!(result, vec!["*.tmp", "target/"]);
        
        // Test with no ignore_patterns key
        let empty_yaml = Value::Mapping(serde_yaml::Mapping::new());
        let result = super::extract_ignore_patterns_from_yaml(&empty_yaml);
        assert!(result.is_empty());
        
        // Test with non-sequence ignore_patterns
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            Value::String("ignore_patterns".to_string()),
            Value::String("not_a_sequence".to_string())
        );
        let yaml = Value::Mapping(mapping);
        
        let result = super::extract_ignore_patterns_from_yaml(&yaml);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_with_verbose_flag() {
        let fs = TestFileSystem::new();
        
        // CRITICAL SAFETY: Change to temp directory to avoid overwriting project files
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&fs.root_path).expect("Failed to change to temp directory");
        
        let config_file = fs.create_test_config("verbose.yml");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "--verbose",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
        
        // CRITICAL SAFETY: Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore original directory");
    }

    #[test]
    fn test_apply_with_overwrite_flag() {
        let fs = TestFileSystem::new();
        
        // CRITICAL SAFETY: Change to temp directory to avoid overwriting project files
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&fs.root_path).expect("Failed to change to temp directory");
        
        let config_file = fs.create_test_config("overwrite.yml");
        
        // Run once to create files
        let args = vec![config_file.to_str().unwrap()];
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
        
        // Run again with overwrite flag
        let args = vec![
            config_file.to_str().unwrap(),
            "--overwrite",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
        
        // CRITICAL SAFETY: Restore original directory BEFORE fs goes out of scope
        std::env::set_current_dir(&original_dir).expect("Failed to restore original directory");
        
        // fs will be dropped here, but we've already restored the directory
    }

    #[test]
    fn test_apply_config_from_matches() {
        let args = vec![
            "test.yml",
            "--overwrite",
            "--verbose",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.input_path.to_str().unwrap(), "test.yml");
            assert!(config.overwrite);
            assert!(config.verbose);
            assert!(!config.dry_run);
        }
    }

    #[test]
    fn test_apply_config_defaults() {
        let args = vec!["basic.yml"];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.input_path.to_str().unwrap(), "basic.yml");
            assert!(!config.overwrite);
            assert!(!config.verbose);
            assert!(!config.dry_run);
        }
    }

    #[test]
    fn test_apply_with_binary_files_and_ignore_patterns() {
        let fs = TestFileSystem::new();
        
        // Create a config with binary_files and ignore_patterns
        let config_content = r#"
directories:
  test_complex:
    hello_main.rs: |
      fn main() {
          println!("Hello, world!");
      }
binary_files:
  - "image.png"
  - "binary.exe"
ignore_patterns:
  - "*.tmp"
  - "target/"
"#;
        let config_file = fs.create_config_from_content("complex.yml", config_content);
        
        let args = vec![
            config_file.to_str().unwrap(),
            "--dry-run",
            "--verbose",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_with_output_directory() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        let output_dir = fs.path("output");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "-o",
            output_dir.to_str().unwrap(),
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.output_dir, output_dir);
            assert!(!config.overwrite);
            assert!(!config.dry_run);
        }
    }

    #[test]
    fn test_apply_with_long_output_flag() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        let output_dir = fs.path("output");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.output_dir, output_dir);
        }
    }

    #[test]
    fn test_apply_output_defaults_to_current_dir() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        
        let args = vec![config_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.output_dir, std::path::PathBuf::from("."));
        }
    }

    #[test]
    fn test_apply_overwrite_flag_is_separate_from_output() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        let output_dir = fs.path("output");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "-o",
            output_dir.to_str().unwrap(),
            "--overwrite",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            let config = super::ApplyConfig::from_matches(&sub_m);
            assert_eq!(config.output_dir, output_dir);
            assert!(config.overwrite);
        }
    }
}
