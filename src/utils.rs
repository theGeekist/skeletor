//! Shared utility functions for reducing code duplication across modules
//!
//! This module provides common operations used by multiple modules:
//! - File I/O with consistent error handling
//! - YAML parsing with proper error conversion
//! - Output formatting utilities

use crate::errors::SkeletorError;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// Read a file to string with consistent error handling
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, SkeletorError> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| SkeletorError::from_io_with_context(e, path.to_path_buf()))
}

/// Write string to file with consistent error handling
pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), SkeletorError> {
    let path = path.as_ref();
    fs::write(path, content)
        .map_err(|e| SkeletorError::from_io_with_context(e, path.to_path_buf()))
}

/// Parse YAML string with consistent error handling
pub fn parse_yaml_string(yaml_str: &str) -> Result<Value, SkeletorError> {
    serde_yaml::from_str(yaml_str)
        .map_err(|e| SkeletorError::invalid_yaml(e.to_string()))
}

/// Read and parse YAML file in one operation
pub fn read_yaml_file<P: AsRef<Path>>(path: P) -> Result<Value, SkeletorError> {
    let content = read_file_to_string(path)?;
    parse_yaml_string(&content)
}

/// Output utilities for consistent formatting
// Note: For consistent output formatting, use the output.rs module's Reporter system
// which provides DefaultReporter and SilentReporter with professional CLI formatting.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::*;

    #[test]
    fn test_read_file_to_string() {
        let fs = TestFileSystem::new();
        let file_path = fs.create_file("test.txt", "Hello, world!");

        let content = read_file_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_write_string_to_file() {
        let fs = TestFileSystem::new();
        let file_path = fs.path("output.txt");

        write_string_to_file(&file_path, "Test content").unwrap();
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_parse_yaml_string() {
        let yaml_str = r#"
        test:
          key: "value"
        "#;

        let result = parse_yaml_string(yaml_str);
        assert!(result.is_ok());
        
        let yaml = result.unwrap();
        if let Value::Mapping(map) = yaml {
            assert!(map.contains_key(&Value::String("test".to_string())));
        } else {
            panic!("Expected YAML mapping");
        }
    }

    #[test]
    fn test_parse_yaml_string_invalid() {
        let invalid_yaml = "invalid: yaml: content: [";
        let result = parse_yaml_string(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_yaml_file() {
        let fs = TestFileSystem::new();
        let yaml_content = r#"
        directories:
          src:
            main.rs: "fn main() {}"
        "#;
        let file_path = fs.create_file("config.yaml", yaml_content);

        let result = read_yaml_file(&file_path);
        assert!(result.is_ok());
    }
}