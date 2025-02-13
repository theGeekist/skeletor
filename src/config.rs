use crate::errors::SkeletorError;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

/// Reads the YAML configuration file and extracts the "directories" key.
/// (Any extra keys are ignored by the apply functionality.)
pub fn read_config(path: &Path) -> Result<Yaml, SkeletorError> {
    let content = fs::read_to_string(path)?;
    let yaml_docs = YamlLoader::load_from_str(&content)?;

    let directories = yaml_docs
        .first()
        .and_then(|doc| doc["directories"].as_hash())
        .ok_or_else(|| SkeletorError::Config("'directories' key is missing or invalid".into()))?;

    Ok(Yaml::Hash(directories.clone()))
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
        if let Yaml::Hash(map) = config {
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
    }
}
