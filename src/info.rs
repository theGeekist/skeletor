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
    use clap::{Arg, Command};
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_info_defaults_to_local_config() {
        // Create a temporary ".skeletorrc" with a valid YAML for test.
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join(".skeletorrc");
        fs::write(
            &config_path,
            r#"
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
"#,
        )
        .unwrap();
        let orig_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let args = vec!["skeletor", "info"];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("Specify the YAML configuration file")
                        .index(1),
                ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("info") {
            let result = run_info(sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Info subcommand not found");
        }
        env::set_current_dir(orig_dir).unwrap();
    }

    #[test]
    fn test_run_info_with_input() {
        // Create a temporary configuration file with a valid YAML for test.
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(
            &config_path,
            r#"
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
"#,
        )
        .unwrap();

        let args = vec!["skeletor", "info", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("Specify the YAML configuration file")
                        .index(1),
                ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("info") {
            let result = run_info(sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_missing_file() {
        let args = vec!["skeletor", "info", "missing.yaml"];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("Specify the YAML configuration file")
                        .index(1),
                ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("info") {
            let result = run_info(sub_m);
            assert!(result.is_err());
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_invalid_yaml() {
        // Create a temporary configuration file with invalid YAML for test.
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(
            &config_path,
            "invalid_yaml: data\n\tbad_indent: - missing_value",
        )
        .unwrap();

        let args = vec!["skeletor", "info", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("Specify the YAML configuration file")
                        .index(1),
                ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("info") {
            let result = run_info(sub_m);
            assert!(result.is_err());
        } else {
            panic!("Info subcommand not found");
        }
    }

    #[test]
    fn test_run_info_with_stats_and_blacklist() {
        // Create a temporary configuration file with stats and blacklist for test.
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(
            &config_path,
            r#"
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
"#,
        )
        .unwrap();

        let args = vec!["skeletor", "info", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("Specify the YAML configuration file")
                        .index(1),
                ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("info") {
            let result = run_info(sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Info subcommand not found");
        }
    }
}
