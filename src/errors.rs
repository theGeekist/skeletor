use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkeletorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}
