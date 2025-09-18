//! Shared test utilities for reducing duplication across test modules
//! 
//! This module provides common helper functions for:
//! - CLI argument parsing and testing
//! - Temporary file/directory creation
//! - YAML configuration setup
//! - Test assertion helpers

#[cfg(test)]
pub mod helpers {
    use clap::ArgMatches;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{TempDir, tempdir};

    /// Helper for creating CLI matches for a given subcommand with arguments
    pub fn create_cli_matches_for_subcommand(subcommand: &str, args: Vec<&str>) -> Option<ArgMatches> {
        let mut full_args = vec!["skeletor", subcommand];
        full_args.extend(args);
        
        let matches = crate::build_cli().get_matches_from(full_args);
        matches.subcommand_matches(subcommand).cloned()
    }

    /// Helper for creating CLI matches for apply subcommand
    pub fn create_apply_matches(args: Vec<&str>) -> Option<ArgMatches> {
        create_cli_matches_for_subcommand("apply", args)
    }

    /// Helper for creating CLI matches for snapshot subcommand  
    pub fn create_snapshot_matches(args: Vec<&str>) -> Option<ArgMatches> {
        create_cli_matches_for_subcommand("snapshot", args)
    }

    /// Helper for creating CLI matches for info subcommand
    pub fn create_info_matches(args: Vec<&str>) -> Option<ArgMatches> {
        create_cli_matches_for_subcommand("info", args)
    }

    /// Create a temporary directory with test files
    pub struct TestFileSystem {
        #[allow(dead_code)]
        pub temp_dir: TempDir,
        pub root_path: PathBuf,
    }

    impl Default for TestFileSystem {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TestFileSystem {
        pub fn new() -> Self {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let root_path = temp_dir.path().to_path_buf();
            
            Self { temp_dir, root_path }
        }

        /// Create a file with given content at the specified path (relative to temp dir)
        pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> PathBuf {
            let full_path = self.root_path.join(path);
            
            // Create parent directories if they don't exist
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directories");
            }
            
            fs::write(&full_path, content).expect("Failed to write file");
            full_path
        }

        /// Create a binary file with given content at the specified path (relative to temp dir)
        pub fn create_binary_file<P: AsRef<Path>>(&self, path: P, content: &[u8]) -> PathBuf {
            let full_path = self.root_path.join(path);
            
            // Create parent directories if they don't exist
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directories");
            }
            
            fs::write(&full_path, content).expect("Failed to write binary file");
            full_path
        }

        /// Create a directory at the specified path (relative to temp dir)
        #[allow(dead_code)]
        pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
            let full_path = self.root_path.join(path);
            fs::create_dir_all(&full_path).expect("Failed to create directory");
            full_path
        }

        /// Create a standard YAML config file for testing
        pub fn create_test_config(&self, filename: &str) -> PathBuf {
            let config_content = r#"
directories:
  test_output:
    hello.rs: |
      fn main() {
          println!("Hello, world!");
      }
    module.rs: ""
  test_files:
    sample.rs: "// Test content"
"#;
            self.create_file(filename, config_content)
        }

        /// Create an invalid YAML config file for testing error cases
        pub fn create_invalid_config(&self, filename: &str) -> PathBuf {
            let invalid_content = "invalid: yaml: content: [";
            self.create_file(filename, invalid_content)
        }

        /// Create a config file without directories key
        pub fn create_config_without_directories(&self, filename: &str) -> PathBuf {
            let content = "other_key: value\nmetadata: test";
            self.create_file(filename, content)
        }

        /// Create a config file with custom content
        pub fn create_config_from_content(&self, filename: &str, content: &str) -> PathBuf {
            self.create_file(filename, content)
        }

        /// Create a basic project structure for snapshot testing
        #[allow(dead_code)]
        pub fn create_sample_project(&self) -> PathBuf {
            self.create_dir("sample_project");
            self.create_file("sample_project/main_file.rs", "fn main() { println!(\"Hello!\"); }");
            self.create_file("sample_project/lib_file.rs", "// Library code");
            self.create_file(".hidden", "secret");
            self.create_file("README.md", "# Test Project");
            
            self.root_path.clone()
        }

        /// Get a path relative to the temp directory  
        pub fn path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
            self.root_path.join(relative_path)
        }
    }

    /// Assert that a CLI command execution succeeds
    pub fn assert_command_succeeds<F>(command_fn: F) 
    where 
        F: FnOnce() -> Result<(), crate::errors::SkeletorError>
    {
        let result = command_fn();
        if let Err(e) = &result {
            panic!("Command should have succeeded but failed with: {}", e);
        }
        assert!(result.is_ok());
    }

    /// Assert that a CLI command execution fails
    pub fn assert_command_fails<F>(command_fn: F) 
    where 
        F: FnOnce() -> Result<(), crate::errors::SkeletorError>
    {
        let result = command_fn();
        assert!(result.is_err(), "Command should have failed but succeeded");
    }

    /// Assert that a file exists and has expected content
    #[allow(dead_code)]
    pub fn assert_file_content<P: AsRef<Path>>(path: P, expected_content: &str) {
        let path = path.as_ref();
        assert!(path.exists(), "File should exist: {}", path.display());
        
        let actual_content = fs::read_to_string(path)
            .expect("Failed to read file content");
        assert_eq!(actual_content, expected_content, 
                   "File content mismatch for: {}", path.display());
    }

    /// Assert that a directory exists
    #[allow(dead_code)]
    pub fn assert_dir_exists<P: AsRef<Path>>(path: P) {
        let path = path.as_ref();
        assert!(path.exists(), "Directory should exist: {}", path.display());
        assert!(path.is_dir(), "Path should be a directory: {}", path.display());
    }
}