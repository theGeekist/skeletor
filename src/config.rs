use crate::errors::SkeletorError;
use serde_yaml::Value;
use std::fs;
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
    pub blacklist: Option<Vec<String>>,
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
        let yaml_doc: Value = serde_yaml::from_str(yaml)
            .map_err(|e| SkeletorError::invalid_yaml(e.to_string()))?;

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
        let content = fs::read_to_string(path)
            .map_err(|e| SkeletorError::from_io_with_context(e, path.to_path_buf()))?;
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
            blacklist: yaml_doc.get("blacklist").and_then(|v| {
                v.as_sequence()?.iter()
                    .map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Option<Vec<_>>>()
            }),
        })
    }
}

pub fn read_config(path: &Path) -> Result<Value, SkeletorError> {
    let content = fs::read_to_string(path)
        .map_err(|e| SkeletorError::from_io_with_context(e, path.to_path_buf()))?;
    let yaml_doc: Value = serde_yaml::from_str(&content)
        .map_err(|e| SkeletorError::invalid_yaml(e.to_string()))?;

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
    use tempfile::tempdir;

    #[test]
    fn test_default_file_path_when_input_not_provided() {
        // When no input is specified, default_file_path returns ".skeletorrc"
        let path = default_file_path(None);
        assert_eq!(path, PathBuf::from(".skeletorrc"));
    }

    #[test]
    fn test_read_config_invalid() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("invalid.yaml");

        let invalid_yaml_content = "invalid_yaml: data\n\tbad_indent: - missing_value";
        fs::write(&config_file, invalid_yaml_content).unwrap();

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

        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.yaml");
        fs::write(&test_file, yaml_str).unwrap();

        let config = read_config(&test_file).unwrap();

        if let Value::Mapping(map) = config {
            assert!(map.contains_key(Value::String("src".to_string())));
        } else {
            panic!("Expected a YAML mapping");
        }
    }
}
