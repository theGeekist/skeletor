use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkeletorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    
    // User-friendly error variants
    #[error("file not found: '{path}'\ntip: Check that the file exists and you have read permissions")]
    FileNotFound { path: PathBuf },
    
    #[error("directory not found: '{path}'\ntip: Verify the directory path exists and is accessible")]
    DirectoryNotFound { path: PathBuf },
    
    #[error("permission denied: '{path}'\ntip: Check file/directory permissions or run with appropriate privileges")]
    PermissionDenied { path: PathBuf },
    
    #[error("invalid YAML configuration: {message}\ntip: Validate your YAML syntax using an online YAML validator")]
    InvalidYaml { message: String },
    
    #[error("missing configuration key: '{key}'\ntip: Ensure your YAML file contains the required '{key}' section")]
    MissingConfigKey { key: String },
    
    #[error("invalid ignore pattern: '{pattern}'\ntip: Check glob pattern syntax (e.g., '*.log', 'target/*')")]
    InvalidIgnorePattern { pattern: String },
}

impl SkeletorError {
    /// Creates a contextual IO error based on the operation and path
    pub fn from_io_with_context(error: io::Error, path: PathBuf) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => {
                // For directories, we can check the path or context
                // If it doesn't end with an extension and doesn't exist, likely a directory
                let path_str = path.to_string_lossy();
                if path_str.ends_with('/') || 
                   (!path_str.contains('.') && path.extension().is_none()) ||
                   path.is_dir() {
                    Self::DirectoryNotFound { path }
                } else {
                    Self::FileNotFound { path }
                }
            }
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            _ => Self::Io(error), // Fallback to generic IO error
        }
    }
    
    /// Creates a directory not found error specifically
    pub fn directory_not_found(path: PathBuf) -> Self {
        Self::DirectoryNotFound { path }
    }
    
    /// Creates a user-friendly YAML error
    pub fn invalid_yaml(message: impl Into<String>) -> Self {
        Self::InvalidYaml { message: message.into() }
    }
    
    /// Creates a missing config key error
    pub fn missing_config_key(key: impl Into<String>) -> Self {
        Self::MissingConfigKey { key: key.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_file_not_found_error() {
        let path = PathBuf::from("test.txt");
        let error = SkeletorError::FileNotFound { path: path.clone() };
        let error_str = error.to_string();
        assert!(error_str.contains("test.txt"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("Check that the file exists"));
    }

    #[test]
    fn test_directory_not_found_error() {
        let path = PathBuf::from("test_dir");
        let error = SkeletorError::DirectoryNotFound { path: path.clone() };
        let error_str = error.to_string();
        assert!(error_str.contains("test_dir"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("Verify the directory path"));
    }

    #[test]
    fn test_permission_denied_error() {
        let path = PathBuf::from("/restricted/file.txt");
        let error = SkeletorError::PermissionDenied { path: path.clone() };
        let error_str = error.to_string();
        assert!(error_str.contains("/restricted/file.txt"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("Check file/directory permissions"));
    }

    #[test]
    fn test_invalid_yaml_error() {
        let error = SkeletorError::InvalidYaml { 
            message: "unexpected character".to_string() 
        };
        let error_str = error.to_string();
        assert!(error_str.contains("unexpected character"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("YAML validator"));
    }

    #[test]
    fn test_missing_config_key_error() {
        let error = SkeletorError::MissingConfigKey { 
            key: "directories".to_string() 
        };
        let error_str = error.to_string();
        assert!(error_str.contains("directories"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("required 'directories' section"));
    }

    #[test]
    fn test_invalid_ignore_pattern_error() {
        let error = SkeletorError::InvalidIgnorePattern { 
            pattern: "[invalid".to_string() 
        };
        let error_str = error.to_string();
        assert!(error_str.contains("[invalid"));
        assert!(error_str.contains("tip:"));
        assert!(error_str.contains("glob pattern syntax"));
    }

    #[test]
    fn test_from_io_with_context_file_not_found() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let path = PathBuf::from("missing.txt");
        let error = SkeletorError::from_io_with_context(io_error, path);
        
        match error {
            SkeletorError::FileNotFound { path } => {
                assert_eq!(path, PathBuf::from("missing.txt"));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_from_io_with_context_directory_not_found() {
        let io_error = IoError::new(ErrorKind::NotFound, "directory not found");
        let path = PathBuf::from("missing_dir/");
        let error = SkeletorError::from_io_with_context(io_error, path);
        
        match error {
            SkeletorError::DirectoryNotFound { path } => {
                assert_eq!(path, PathBuf::from("missing_dir/"));
            }
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_from_io_with_context_permission_denied() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "permission denied");
        let path = PathBuf::from("/restricted");
        let error = SkeletorError::from_io_with_context(io_error, path);
        
        match error {
            SkeletorError::PermissionDenied { path } => {
                assert_eq!(path, PathBuf::from("/restricted"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn test_from_io_with_context_other_error() {
        let io_error = IoError::new(ErrorKind::InvalidData, "invalid data");
        let path = PathBuf::from("test.txt");
        let error = SkeletorError::from_io_with_context(io_error, path);
        
        match error {
            SkeletorError::Io(_) => {}, // Should fall back to generic IO error
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_directory_not_found_helper() {
        let path = PathBuf::from("test_dir");
        let error = SkeletorError::directory_not_found(path.clone());
        
        match error {
            SkeletorError::DirectoryNotFound { path: err_path } => {
                assert_eq!(err_path, path);
            }
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_invalid_yaml_helper() {
        let error = SkeletorError::invalid_yaml("syntax error");
        
        match error {
            SkeletorError::InvalidYaml { message } => {
                assert_eq!(message, "syntax error");
            }
            _ => panic!("Expected InvalidYaml error"),
        }
    }

    #[test]
    fn test_missing_config_key_helper() {
        let error = SkeletorError::missing_config_key("directories");
        
        match error {
            SkeletorError::MissingConfigKey { key } => {
                assert_eq!(key, "directories");
            }
            _ => panic!("Expected MissingConfigKey error"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = SkeletorError::Config("test config error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test config error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::InvalidData, "test io error");
        let error: SkeletorError = io_error.into();
        
        match error {
            SkeletorError::Io(_) => {},
            _ => panic!("Expected Io error from conversion"),
        }
    }

    #[test]
    fn test_yaml_error_conversion() {
        // Create a YAML error by parsing invalid YAML
        let yaml_result: Result<serde_yaml::Value, serde_yaml::Error> = 
            serde_yaml::from_str("invalid: yaml: [");
        
        if let Err(yaml_error) = yaml_result {
            let error: SkeletorError = yaml_error.into();
            match error {
                SkeletorError::Yaml(_) => {},
                _ => panic!("Expected Yaml error from conversion"),
            }
        } else {
            panic!("Expected YAML parsing to fail");
        }
    }
}
