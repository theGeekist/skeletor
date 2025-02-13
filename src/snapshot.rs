use crate::config::default_file_path;
use crate::errors::SkeletorError;
use crate::tasks::{compute_stats, traverse_directory};
use clap::ArgMatches;
use linked_hash_map::LinkedHashMap;
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

// For timestamp formatting.
use chrono::Utc;
// For glob-style ignoring.
use globset::{Glob, GlobSetBuilder};

/// Runs the snapshot subcommand. Generates a snapshot with extra annotations, stats, and ignore support.
pub fn run_snapshot(matches: &ArgMatches) -> Result<(), SkeletorError> {
    let source_path = PathBuf::from(matches.get_one::<String>("source").unwrap());
    // Use default_file_path for output: if nothing is provided, default to ".skeletorrc"
    let output_path = Some(default_file_path(matches.get_one::<String>("output")));
    let include_contents = *matches
        .get_one::<bool>("include_contents")
        .unwrap_or(&false);
    let dry_run = *matches.get_one::<bool>("dry_run").unwrap_or(&false);
    let user_note = matches.get_one::<String>("note").map(|s| s.to_string());

    info!("Taking snapshot of folder: {:?}", source_path);

    // Process ignore flags.
    let mut ignore_patterns: Vec<String> = Vec::new();
    if let Some(vals) = matches.get_many::<String>("ignore") {
        for val in vals {
            let val = val.as_str();
            let candidate = Path::new(val);
            if candidate.exists() && candidate.is_file() {
                // Read file and add non-empty, non-comment lines.
                let content = fs::read_to_string(candidate)?;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    ignore_patterns.push(trimmed.to_string());
                }
            } else {
                // Treat the value directly as a glob pattern.
                ignore_patterns.push(val.to_string());
            }
        }
    }

    // Build the GlobSet if any ignore patterns are provided.
    let globset = if !ignore_patterns.is_empty() {
        let mut builder = GlobSetBuilder::new();
        for pat in &ignore_patterns {
            let pat = pat.trim();
            builder.add(Glob::new(pat).map_err(|e| SkeletorError::Config(e.to_string()))?);
        }
        Some(
            builder
                .build()
                .map_err(|e| SkeletorError::Config(e.to_string()))?,
        )
    } else {
        None
    };

    // Take the snapshot.
    let (dir_snapshot, binary_files) =
        traverse_directory(&source_path, include_contents, globset.as_ref())?;

    // Compute stats.
    let (files_count, dirs_count) = compute_stats(&dir_snapshot);

    let now = Utc::now().to_rfc3339();
    // Preserve the "created" timestamp if output file exists.
    let created = if let Some(ref out_file) = output_path {
        if out_file.exists() {
            let existing = fs::read_to_string(out_file)?;
            let docs = YamlLoader::load_from_str(&existing)?;
            if let Some(c) = docs.first().and_then(|doc| doc["created"].as_str()) {
                c.to_string()
            } else {
                now.clone()
            }
        } else {
            now.clone()
        }
    } else {
        now.clone()
    };
    let updated = now;

    let mut auto_info = format!("Snapshot generated from folder: {:?}", source_path);
    if binary_files.is_empty() {
        auto_info.push_str("\nNo binary files detected.");
    } else {
        auto_info.push_str(&format!(
            "\nBinary files detected (contents omitted): {:?}",
            binary_files
        ));
    }

    // Build the top-level YAML mapping without changing the original structure.
    let mut top_map = LinkedHashMap::new(); // Use LinkedHashMap instead of BTreeMap
    top_map.insert(Yaml::String("created".into()), Yaml::String(created));
    top_map.insert(Yaml::String("updated".into()), Yaml::String(updated));
    top_map.insert(
        Yaml::String("generated_comments".into()),
        Yaml::String(auto_info),
    );
    if let Some(note) = user_note {
        top_map.insert(Yaml::String("notes".into()), Yaml::String(note));
    }
    if !ignore_patterns.is_empty() {
        let patterns_yaml: Vec<Yaml> = ignore_patterns.into_iter().map(Yaml::String).collect();
        top_map.insert(Yaml::String("blacklist".into()), Yaml::Array(patterns_yaml));
    }
    // Add stats.
    let mut stats_map = LinkedHashMap::new();
    stats_map.insert(
        Yaml::String("files".into()),
        Yaml::String(files_count.to_string()),
    );
    stats_map.insert(
        Yaml::String("directories".into()),
        Yaml::String(dirs_count.to_string()),
    );
    top_map.insert(Yaml::String("stats".into()), Yaml::Hash(stats_map));
    top_map.insert(Yaml::String("directories".into()), dir_snapshot);
    let snapshot_yaml = Yaml::Hash(top_map);

    // Emit the YAML.
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&snapshot_yaml).unwrap();
    }

    if dry_run {
        println!("Dry run enabled. The following snapshot would be generated:");
        println!("{}", out_str);
    } else if let Some(out_file) = output_path {
        fs::write(&out_file, out_str.clone())?;
        println!("Snapshot written to {:?}", out_file);
    } else {
        println!("{}", out_str);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};
    use tempfile::tempdir;
    use yaml_rust::YamlLoader;

    #[test]
    fn test_snapshot_directory_without_contents() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(&test_dir, false, None).unwrap();

        if let Yaml::Hash(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_snapshot_directory_with_contents() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(&test_dir, true, None).unwrap();

        if let Yaml::Hash(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_compute_stats() {
        let yaml_str = r#"
        src:
          index.js: "console.log('Hello, world!');"
          components:
            Header.js: "// Header component"
        "#;
        let yaml = YamlLoader::load_from_str(yaml_str).unwrap()[0].clone();

        let (files, dirs) = compute_stats(&yaml);

        assert_eq!(files, 2);
        assert_eq!(dirs, 2); // One for "src" and one for "components"
    }

    #[test]
    fn test_traverse_directory() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure with a hidden file and a regular file.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
        // Hidden file should be included.
        fs::write(src.join(".hidden.txt"), "secret").unwrap();

        let (yaml_structure, binaries) = traverse_directory(&test_dir, false, None).unwrap();

        if let Yaml::Hash(map) = yaml_structure {
            // Expect "src" key exists.
            assert!(map.contains_key(&Yaml::String("src".into())));
        } else {
            panic!("Expected a YAML hash");
        }
        // Since we are not including contents, binaries should be empty.
        assert!(binaries.is_empty());
    }

    #[test]
    fn test_run_snapshot_with_dry_run() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec!["skeletor", "snapshot", test_dir.to_str().unwrap(), "--dry-run"];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("snapshot")
                    .arg(
                        Arg::new("source")
                            .value_name("FOLDER")
                            .help("The source folder to snapshot")
                            .required(true),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .value_name("FILE")
                            .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                    )
                    .arg(
                        Arg::new("include_contents")
                            .long("include-contents")
                            .help("Include file contents in the snapshot (for text files; binary files will be empty)")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("ignore")
                            .short('I')
                            .long("ignore")
                            .value_name("PATTERN_OR_FILE")
                            .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times.")
                            .action(ArgAction::Append),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .short('d')
                            .long("dry-run")
                            .help("Perform a trial run with no changes made")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("note")
                            .short('n')
                            .long("note")
                            .value_name("NOTE")
                            .help("Optional user note to include in the snapshot"),
                    ),
            )
            .get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("snapshot") {
            let result = run_snapshot(sub_m);
            assert!(result.is_ok());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_run_snapshot_with_output() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let output_file = temp_dir.path().join("output.yaml");

        // Create a simple structure.
        let src = test_dir.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

        let args = vec![
            "skeletor",
            "snapshot",
            test_dir.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("snapshot")
                    .arg(
                        Arg::new("source")
                            .value_name("FOLDER")
                            .help("The source folder to snapshot")
                            .required(true),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .value_name("FILE")
                            .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                    )
                    .arg(
                        Arg::new("include_contents")
                            .long("include-contents")
                            .help("Include file contents in the snapshot (for text files; binary files will be empty)")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("ignore")
                            .short('I')
                            .long("ignore")
                            .value_name("PATTERN_OR_FILE")
                            .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times.")
                            .action(ArgAction::Append),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .short('d')
                            .long("dry-run")
                            .help("Perform a trial run with no changes made")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("note")
                            .short('n')
                            .long("note")
                            .value_name("NOTE")
                            .help("Optional user note to include in the snapshot"),
                    ),
            )
            .get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("snapshot") {
            let result = run_snapshot(sub_m);
            assert!(result.is_ok());
            assert!(output_file.exists());
        } else {
            panic!("Snapshot subcommand not found");
        }
    }
    
    #[test]
    fn test_run_snapshot_with_ignore_patterns() {
      let temp_dir = tempdir().unwrap();
      let test_dir = temp_dir.path();

      // Create a simple structure.
      let src = test_dir.join("src");
      fs::create_dir(&src).unwrap();
      fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
      fs::write(src.join("ignore.txt"), "ignore me").unwrap();

      let ignore_file = temp_dir.path().join("ignore_patterns.txt");
      fs::write(&ignore_file, "ignore.txt").unwrap();

      let args = vec![
          "skeletor",
          "snapshot",
          test_dir.to_str().unwrap(),
          "--ignore",
          ignore_file.to_str().unwrap(),
      ];
      let matches = Command::new("Skeletor")
          .subcommand(
              Command::new("snapshot")
                  .arg(
                      Arg::new("source")
                          .value_name("FOLDER")
                          .help("The source folder to snapshot")
                          .required(true),
                  )
                  .arg(
                      Arg::new("output")
                          .short('o')
                          .long("output")
                          .value_name("FILE")
                          .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                  )
                  .arg(
                      Arg::new("include_contents")
                          .long("include-contents")
                          .help("Include file contents in the snapshot (for text files; binary files will be empty)")
                          .action(ArgAction::SetTrue),
                  )
                  .arg(
                      Arg::new("ignore")
                          .short('I')
                          .long("ignore")
                          .value_name("PATTERN_OR_FILE")
                          .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times.")
                          .action(ArgAction::Append),
                  )
                  .arg(
                      Arg::new("dry_run")
                          .short('d')
                          .long("dry-run")
                          .help("Perform a trial run with no changes made")
                          .action(ArgAction::SetTrue),
                  )
                  .arg(
                      Arg::new("note")
                          .short('n')
                          .long("note")
                          .value_name("NOTE")
                          .help("Optional user note to include in the snapshot"),
                  ),
          )
          .get_matches_from(args);

      if let Some(sub_m) = matches.subcommand_matches("snapshot") {
          let result = run_snapshot(sub_m);
          assert!(result.is_ok());
      } else {
          panic!("Snapshot subcommand not found");
      }
  }

    #[test]
    fn test_run_snapshot_with_binary_files() {
      let temp_dir = tempdir().unwrap();
      let test_dir = temp_dir.path();

      // Create a simple structure with a binary file.
      let src = test_dir.join("src");
      fs::create_dir(&src).unwrap();
      fs::write(src.join("index.js"), "console.log('Hello');").unwrap();
      fs::write(src.join("binary.bin"), &[0, 159, 146, 150]).unwrap();

      let args = vec!["skeletor", "snapshot", test_dir.to_str().unwrap()];
      let matches = Command::new("Skeletor")
          .subcommand(
              Command::new("snapshot")
                  .arg(
                      Arg::new("source")
                          .value_name("FOLDER")
                          .help("The source folder to snapshot")
                          .required(true),
                  )
                  .arg(
                      Arg::new("output")
                          .short('o')
                          .long("output")
                          .value_name("FILE")
                          .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                  )
                  .arg(
                      Arg::new("include_contents")
                          .long("include-contents")
                          .help("Include file contents in the snapshot (for text files; binary files will be empty)")
                          .action(ArgAction::SetTrue),
                  )
                  .arg(
                      Arg::new("ignore")
                          .short('I')
                          .long("ignore")
                          .value_name("PATTERN_OR_FILE")
                          .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times.")
                          .action(ArgAction::Append),
                  )
                  .arg(
                      Arg::new("dry_run")
                          .short('d')
                          .long("dry-run")
                          .help("Perform a trial run with no changes made")
                          .action(ArgAction::SetTrue),
                  )
                  .arg(
                      Arg::new("note")
                          .short('n')
                          .long("note")
                          .value_name("NOTE")
                          .help("Optional user note to include in the snapshot"),
                  ),
          )
          .get_matches_from(args);

      if let Some(sub_m) = matches.subcommand_matches("snapshot") {
          let result = run_snapshot(sub_m);
          assert!(result.is_ok());
      } else {
          panic!("Snapshot subcommand not found");
      }
  }

  #[test]
  fn test_run_snapshot_with_notes() {
      let temp_dir = tempdir().unwrap();
      let test_dir = temp_dir.path();
      let output_file = temp_dir.path().join("output.yaml");

      // Create a simple structure.
      let src = test_dir.join("src");
      fs::create_dir(&src).unwrap();
      fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

      let args = vec![
          "skeletor",
          "snapshot",
          test_dir.to_str().unwrap(),
          "--output",
          output_file.to_str().unwrap(),
          "--note",
          "This is a test note",
      ];
      let matches = Command::new("Skeletor")
          .subcommand(
              Command::new("snapshot")
                  .arg(
                      Arg::new("source")
                          .value_name("FOLDER")
                          .help("The source folder to snapshot")
                          .required(true),
                  )
                  .arg(
                      Arg::new("output")
                          .short('o')
                          .long("output")
                          .value_name("FILE")
                          .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                  )
                  .arg(
                      Arg::new("include_contents")
                          .long("include-contents")
                          .help("Include file contents in the snapshot (for text files; binary files will be empty)"),
                  )
                  .arg(
                      Arg::new("ignore")
                          .short('I')
                          .long("ignore")
                          .value_name("PATTERN_OR_FILE")
                          .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times."),
                  )
                  .arg(
                      Arg::new("dry_run")
                          .short('d')
                          .long("dry-run")
                          .help("Perform a trial run with no changes made"),
                  )
                  .arg(
                      Arg::new("note")
                          .short('n')
                          .long("note")
                          .value_name("NOTE")
                          .help("Optional user note to include in the snapshot"),
                  ),
          )
          .get_matches_from(args);

      if let Some(sub_m) = matches.subcommand_matches("snapshot") {
          let result = run_snapshot(sub_m);
          assert!(result.is_ok());
          assert!(output_file.exists());

          // Verify that the note is included in the output file.
          let output_content = fs::read_to_string(output_file).unwrap();
          assert!(output_content.contains("This is a test note"));
      } else {
          panic!("Snapshot subcommand not found");
      }
  }

  #[test]
  fn test_run_snapshot_with_existing_output_file() {
      let temp_dir = tempdir().unwrap();
      let test_dir = temp_dir.path();
      let output_file = temp_dir.path().join("output.yaml");

      // Create a simple structure.
      let src = test_dir.join("src");
      fs::create_dir(&src).unwrap();
      fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

      // Create an existing output file with a "created" timestamp.
      fs::write(
          &output_file,
          r#"
created: "2020-01-01T00:00:00Z"
updated: "2020-01-02T00:00:00Z"
generated_comments: "Test comment"
directories:
src:
  main.rs: "fn main() {}"
"#,
      )
      .unwrap();

      let args = vec![
          "skeletor",
          "snapshot",
          test_dir.to_str().unwrap(),
          "--output",
          output_file.to_str().unwrap(),
      ];
      let matches = Command::new("Skeletor")
          .subcommand(
              Command::new("snapshot")
                  .arg(
                      Arg::new("source")
                          .value_name("FOLDER")
                          .help("The source folder to snapshot")
                          .required(true),
                  )
                  .arg(
                      Arg::new("output")
                          .short('o')
                          .long("output")
                          .value_name("FILE")
                          .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                  )
                  .arg(
                      Arg::new("include_contents")
                          .long("include-contents")
                          .help("Include file contents in the snapshot (for text files; binary files will be empty)"),
                  )
                  .arg(
                      Arg::new("ignore")
                          .short('I')
                          .long("ignore")
                          .value_name("PATTERN_OR_FILE")
                          .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times."),
                  )
                  .arg(
                      Arg::new("dry_run")
                          .short('d')
                          .long("dry-run")
                          .help("Perform a trial run with no changes made"),
                  )
                  .arg(
                      Arg::new("note")
                          .short('n')
                          .long("note")
                          .value_name("NOTE")
                          .help("Optional user note to include in the snapshot"),
                  ),
          )
          .get_matches_from(args);

      if let Some(sub_m) = matches.subcommand_matches("snapshot") {
          let result = run_snapshot(sub_m);
          assert!(result.is_ok());
      } else {
          panic!("Snapshot subcommand not found");
      }
  }

  #[test]
  fn test_run_snapshot_with_final_println() {
      let temp_dir = tempdir().unwrap();
      let test_dir = temp_dir.path();

      // Create a simple structure.
      let src = test_dir.join("src");
      fs::create_dir(&src).unwrap();
      fs::write(src.join("index.js"), "console.log('Hello');").unwrap();

      let args = vec!["skeletor", "snapshot", test_dir.to_str().unwrap()];
      let matches = Command::new("Skeletor")
          .subcommand(
              Command::new("snapshot")
                  .arg(
                      Arg::new("source")
                          .value_name("FOLDER")
                          .help("The source folder to snapshot")
                          .required(true),
                  )
                  .arg(
                      Arg::new("output")
                          .short('o')
                          .long("output")
                          .value_name("FILE")
                          .help("Output file for the generated snapshot YAML (prints to stdout if not provided)"),
                  )
                  .arg(
                      Arg::new("include_contents")
                          .long("include-contents")
                          .help("Include file contents in the snapshot (for text files; binary files will be empty)"),
                  )
                  .arg(
                      Arg::new("ignore")
                          .short('I')
                          .long("ignore")
                          .value_name("PATTERN_OR_FILE")
                          .help("A glob pattern or a file containing .gitignore style patterns. Can be used multiple times."),
                  )
                  .arg(
                      Arg::new("dry_run")
                          .short('d')
                          .long("dry-run")
                          .help("Perform a trial run with no changes made"),
                  )
                  .arg(
                      Arg::new("note")
                          .short('n')
                          .long("note")
                          .value_name("NOTE")
                          .help("Optional user note to include in the snapshot"),
                  ),
          )
          .get_matches_from(args);

      if let Some(sub_m) = matches.subcommand_matches("snapshot") {
          let result = run_snapshot(sub_m);
          assert!(result.is_ok());
      } else {
          panic!("Snapshot subcommand not found");
      }
  }
}