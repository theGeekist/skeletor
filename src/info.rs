use crate::config::default_file_path;
use crate::errors::SkeletorError;
use clap::ArgMatches;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

/// Runs the info subcommand: prints annotation and stats information from a .skeletorrc file.
pub fn run_info(matches: &ArgMatches) -> Result<(), SkeletorError> {
    // Use default_file_path so that .skeletorrc is used by default.
    let input_path = default_file_path(matches.get_one::<String>("input"));

    let content = fs::read_to_string(&input_path)?;
    let yaml_docs = YamlLoader::load_from_str(&content)?;
    let doc = &yaml_docs[0];

    println!("Information from {:?}:", input_path);
    if let Some(created) = doc["created"].as_str() {
        println!("  Created: {}", created);
    } else {
        println!("  No created timestamp available.");
    }
    if let Some(updated) = doc["updated"].as_str() {
        println!("  Updated: {}", updated);
    } else {
        println!("  No updated timestamp available.");
    }
    if let Some(gen_comments) = doc["generated_comments"].as_str() {
        println!("  Generated comments: {}", gen_comments);
    } else {
        println!("  No generated comments available.");
    }
    if let Some(notes) = doc["notes"].as_str() {
        println!("  User notes: {}", notes);
    } else {
        println!("  No user notes available.");
    }
    if let Some(stats) = doc["stats"].as_hash() {
        let files = stats
            .get(&Yaml::String("files".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let directories = stats
            .get(&Yaml::String("directories".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        println!("  Stats: {} files, {} directories", files, directories);
    } else {
        println!("  No stats available.");
    }
    if let Some(blacklist) = doc["blacklist"].as_vec() {
        let patterns: Vec<&str> = blacklist.iter().filter_map(|p| p.as_str()).collect();
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
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file"),
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

        let args = vec!["skeletor", "info", "--input", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file"),
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
        let args = vec!["skeletor", "info", "--input", "missing.yaml"];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file"),
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
        fs::write(&config_path, "invalid_yaml: data\n\tbad_indent: - missing_value").unwrap();

        let args = vec!["skeletor", "info", "--input", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file"),
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

        let args = vec!["skeletor", "info", "--input", config_path.to_str().unwrap()];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("info").arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file"),
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
