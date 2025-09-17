use crate::config::default_file_path;
use crate::errors::SkeletorError;
use clap::ArgMatches;
use serde_yaml::Value;
use std::fs;

/// Runs the info subcommand: prints annotation and stats information from a .skeletorrc file.
pub fn run_info(matches: &ArgMatches) -> Result<(), SkeletorError> {
    // Use default_file_path so that .skeletorrc is used by default.
    let input_path = default_file_path(matches.get_one::<String>("config"));

    let content = fs::read_to_string(&input_path)
        .map_err(|e| SkeletorError::from_io_with_context(e, input_path.clone()))?;
    let yaml_docs: Value = serde_yaml::from_str(&content)
        .map_err(|e| SkeletorError::invalid_yaml(e.to_string()))?;

    println!("Information from {:?}:", input_path);

    if let Some(created) = yaml_docs.get("created").and_then(Value::as_str) {
        println!("  Created: {}", created);
    } else {
        println!("  No created timestamp available.");
    }

    if let Some(updated) = yaml_docs.get("updated").and_then(Value::as_str) {
        println!("  Updated: {}", updated);
    } else {
        println!("  No updated timestamp available.");
    }

    if let Some(gen_comments) = yaml_docs.get("generated_comments").and_then(Value::as_str) {
        println!("  Generated comments: {}", gen_comments);
    } else {
        println!("  No generated comments available.");
    }

    if let Some(stats) = yaml_docs.get("stats").and_then(Value::as_mapping) {
        let files = stats.get("files").and_then(Value::as_u64).unwrap_or(0);
        let directories = stats
            .get("directories")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        println!("  Stats: {} files, {} directories", files, directories);
    } else {
        println!("  No stats available.");
    }

    if let Some(blacklist) = yaml_docs.get("blacklist").and_then(Value::as_sequence) {
        let patterns: Vec<&str> = blacklist.iter().filter_map(Value::as_str).collect();
        println!("  Blacklist patterns: {:?}", patterns);
    } else {
        println!("  No blacklist information available.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::*;
    use std::env;

    #[test]
    fn test_run_info_defaults_to_local_config() {
        let fs = TestFileSystem::new();
        
        // Create .skeletorrc with metadata
        let config_content = r#"
created: "2020-01-01T00:00:00Z"
updated: "2020-01-02T00:00:00Z"
generated_comments: "Test comment"
directories:
  src:
    main.rs: "fn main() {}"
stats:
  files: "1"
  directories: "1"
blacklist:
  - "*.tmp"
"#;
        let _config_path = fs.create_file(".skeletorrc", config_content);
        
        // Change to temp directory so the test finds .skeletorrc
        let orig_dir = env::current_dir().unwrap();
        env::set_current_dir(&fs.root_path).unwrap();

        let args = vec![];
        if let Some(sub_m) = create_info_matches(args) {
            assert_command_succeeds(|| run_info(&sub_m));
        } else {
            panic!("Info subcommand not found");
        }
        env::set_current_dir(orig_dir).unwrap();
    }

    #[test]
    fn test_run_info_with_input() {
        let fs = TestFileSystem::new();
        let config_path = fs.create_file("config.yaml", r#"
created: "2020-01-01T00:00:00Z"
updated: "2020-01-02T00:00:00Z"
generated_comments: "Test comment"
directories:
  src:
    main.rs: "fn main() {}"
stats:
  files: "1"
  directories: "1"
blacklist:
  - "*.tmp"
"#);

        let args = vec![config_path.to_str().unwrap()];
        if let Some(sub_m) = create_info_matches(args) {
            assert_command_succeeds(|| run_info(&sub_m));
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_missing_file() {
        let args = vec!["missing.yaml"];
        if let Some(sub_m) = create_info_matches(args) {
            assert_command_fails(|| run_info(&sub_m));
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_invalid_yaml() {
        let fs = TestFileSystem::new();
        let config_path = fs.create_invalid_config("config.yaml");

        let args = vec![config_path.to_str().unwrap()];
        if let Some(sub_m) = create_info_matches(args) {
            assert_command_fails(|| run_info(&sub_m));
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_stats_and_blacklist() {
        let fs = TestFileSystem::new();
        let config_path = fs.create_file("config.yaml", r#"
created: "2020-01-01T00:00:00Z"
updated: "2020-01-02T00:00:00Z"
generated_comments: "Test comment"
directories:
  src:
    main.rs: "fn main() {}"
stats:
  files: "1"
  directories: "1"
blacklist:
  - "*.tmp"
"#);

        let args = vec![config_path.to_str().unwrap()];
        if let Some(sub_m) = create_info_matches(args) {
            assert_command_succeeds(|| run_info(&sub_m));
        } else {
            panic!("Info subcommand not found");
        }
    }
}
