use crate::errors::SkeletorError;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

/// Configuration for Skeletor scaffolding operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkeletorConfig {
    pub directories: Value,
    pub metadata: Option<SkeletorMetadata>,
}

/// Metadata associated with a Skeletor configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkeletorMetadata {
    pub created: Option<String>,
    pub updated: Option<String>,
    pub generated_comments: Option<String>,
    pub stats: Option<(usize, usize)>, // (files, directories)
    pub ignore_patterns: Option<Vec<String>>,
}

#[allow(dead_code)]
impl SkeletorConfig {
    /// Create a new configuration from a YAML value
    pub fn new(directories: Value) -> Self {
        Self {
            directories,
            metadata: None,
        }
    }

    /// Create a configuration from a YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, SkeletorError> {
        let yaml_doc: Value = crate::utils::parse_yaml_string(yaml)?;

        let directories = yaml_doc
            .get("directories")
            .ok_or_else(|| SkeletorError::missing_config_key("directories"))?
            .clone();

        let metadata = Self::extract_metadata(&yaml_doc);

        Ok(Self {
            directories,
            metadata,
        })
    }

    /// Create a configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SkeletorError> {
        let path = path.as_ref();
        let content = crate::utils::read_file_to_string(path)?;
        Self::from_yaml_str(&content)
    }

    fn extract_metadata(yaml_doc: &Value) -> Option<SkeletorMetadata> {
        Some(SkeletorMetadata {
            created: yaml_doc.get("created").and_then(|v| v.as_str()).map(|s| s.to_string()),
            updated: yaml_doc.get("updated").and_then(|v| v.as_str()).map(|s| s.to_string()),
            generated_comments: yaml_doc.get("generated_comments").and_then(|v| v.as_str()).map(|s| s.to_string()),
            stats: yaml_doc.get("stats").and_then(|stats| {
                let files = stats.get("files")?.as_u64()? as usize;
                let directories = stats.get("directories")?.as_u64()? as usize;
                Some((files, directories))
            }),
            ignore_patterns: yaml_doc.get("ignore_patterns").and_then(|v| {
                v.as_sequence()?.iter()
                    .map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Option<Vec<_>>>()
            }),
        })
    }
}

pub fn read_config(path: &Path) -> Result<Value, SkeletorError> {
    let yaml_doc: Value = crate::utils::read_yaml_file(path)?;

    let directories = yaml_doc
        .get("directories")
        .and_then(Value::as_mapping)
        .ok_or_else(|| SkeletorError::missing_config_key("directories"))?;

    Ok(Value::Mapping(directories.clone()))
}

/// Returns the provided file path or defaults to ".skeletorrc".
pub fn default_file_path(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        PathBuf::from(path)
    } else {
        PathBuf::from(".skeletorrc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::*;

    #[test]
    fn test_default_file_path_when_input_not_provided() {
        // When no input is specified, default_file_path returns ".skeletorrc"
        let path = default_file_path(None);
        assert_eq!(path, PathBuf::from(".skeletorrc"));
    }

    #[test]
    fn test_default_file_path_with_input() {
        // When input is provided, it should return that path
        let input_string = "custom.yml".to_string();
        let path = default_file_path(Some(&input_string));
        assert_eq!(path, PathBuf::from("custom.yml"));
    }

    #[test]
    fn test_skeletor_config_new() {
        let yaml_value = Value::String("test".to_string());
        let config = SkeletorConfig::new(yaml_value.clone());
        
        assert_eq!(config.directories, yaml_value);
        assert!(config.metadata.is_none());
    }

    #[test]
    fn test_skeletor_config_from_yaml_str_missing_directories() {
        let yaml_str = r#"
        other_field: "value"
        "#;
        
        let result = SkeletorConfig::from_yaml_str(yaml_str);
        assert!(result.is_err());
        
        if let Err(SkeletorError::MissingConfigKey { key, .. }) = result {
            assert_eq!(key, "directories");
        } else {
            panic!("Expected MissingConfigKey error");
        }
    }

    #[test]
    fn test_skeletor_config_from_yaml_str_with_metadata() {
        let yaml_str = r#"
        directories:
          src:
            main.rs: "fn main() {}"
        created: "2023-01-01"
        updated: "2023-01-02"
        generated_comments: "Auto-generated"
        stats:
          files: 5
          directories: 3
        ignore_patterns:
          - "*.tmp"
          - "node_modules"
        "#;
        
        let config = SkeletorConfig::from_yaml_str(yaml_str).unwrap();
        let metadata = config.metadata.unwrap();
        
        assert_eq!(metadata.created, Some("2023-01-01".to_string()));
        assert_eq!(metadata.updated, Some("2023-01-02".to_string()));
        assert_eq!(metadata.generated_comments, Some("Auto-generated".to_string()));
        assert_eq!(metadata.stats, Some((5, 3)));
        assert_eq!(metadata.ignore_patterns, Some(vec!["*.tmp".to_string(), "node_modules".to_string()]));
    }

    #[test]
    fn test_skeletor_config_from_yaml_str_partial_metadata() {
        let yaml_str = r#"
        directories:
          src:
            main.rs: "fn main() {}"
        created: "2023-01-01"
        stats:
          files: 2
        "#;
        
        let config = SkeletorConfig::from_yaml_str(yaml_str).unwrap();
        let metadata = config.metadata.unwrap();
        
        assert_eq!(metadata.created, Some("2023-01-01".to_string()));
        assert_eq!(metadata.updated, None);
        assert_eq!(metadata.stats, None); // Missing directories field
        assert_eq!(metadata.ignore_patterns, None);
    }

    #[test]
    fn test_skeletor_config_from_file_not_found() {
        let result = SkeletorConfig::from_file("nonexistent.yml");
        assert!(result.is_err());
    }

    #[test]
    fn test_skeletor_config_from_file_valid() {
        let fs = TestFileSystem::new();
        let yaml_str = r#"
        directories:
          src:
            main.rs: "fn main() {}"
        "#;
        let config_file = fs.create_file("config.yml", yaml_str);
        
        let config = SkeletorConfig::from_file(&config_file).unwrap();
        assert!(config.directories.is_mapping());
        assert!(config.metadata.is_some());
    }

    #[test]
    fn test_read_config_missing_directories_key() {
        let fs = TestFileSystem::new();
        let yaml_str = r#"
        other_key: "value"
        "#;
        let config_file = fs.create_file("config.yml", yaml_str);
        
        let result = read_config(&config_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_config_invalid() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_invalid_config("invalid.yaml");

        let result = read_config(&config_file);

        assert!(result.is_err());
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

        let fs = TestFileSystem::new();
        let test_file = fs.create_file("test.yaml", yaml_str);

        let config = read_config(&test_file).unwrap();

        if let Value::Mapping(map) = config {
            assert!(map.contains_key(Value::String("src".to_string())));
        } else {
            panic!("Expected a YAML mapping");
        }
    }
}
