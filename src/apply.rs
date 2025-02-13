use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::tasks::{create_files_and_directories, task_path, traverse_structure};
use clap::ArgMatches;
use log::info;
use std::path::Path;
use std::time::Instant;

/// Runs the apply subcommand: reads the YAML config and creates files/directories.
/// In dry-run mode, the tasks are printed without performing any filesystem changes.
pub fn run_apply(matches: &ArgMatches) -> Result<(), SkeletorError> {
    // Use default_file_path so that .skeletorrc is used by default if no input is provided.
    let input_path = default_file_path(matches.get_one::<String>("input"));
    let overwrite = *matches.get_one::<bool>("overwrite").unwrap_or(&false);
    let dry_run = *matches.get_one::<bool>("dry_run").unwrap_or(&false);

    info!("Reading input file: {:?}", input_path);
    info!("Overwrite flag: {:?}", overwrite);

    let config = read_config(&input_path)?;

    if config.is_null() {
        return Err(SkeletorError::Config(
            "'directories' key is required in the YAML file".into(),
        ));
    }

    let start_time = Instant::now();

    let tasks = traverse_structure(Path::new("."), &config);

    if dry_run {
        println!("Dry run enabled. The following tasks would be executed:");
        for task in tasks.iter() {
            println!("{}", task_path(task));
        }
        println!("Dry run complete. No changes were made.");
    } else {
        create_files_and_directories(&tasks, overwrite)?;
        let duration = start_time.elapsed();
        println!(
            "\nSuccessfully generated files and directories in {:?}.",
            duration
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read_config;
    use crate::tasks::traverse_structure;
    use clap::{Arg, ArgAction, Command};
    use std::env;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;
    lazy_static::lazy_static! {
        static ref DIR_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_run_apply_defaults_to_local_config() {
        // Acquire the lock to ensure current_dir modifications are serialized.
        let _lock = DIR_LOCK.lock().unwrap();

        // Create a temporary ".skeletorrc"
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skeletorrc");
        fs::write(
            &config_path,
            r#"
directories:
  src:
    main.rs: "fn main() {}"
"#,
        )
        .unwrap();
        // Back up the current working directory.
        let orig_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Build dummy matches with no input provided.
        let args = vec!["skeletor", "apply"];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("apply")
                    .arg(
                        Arg::new("input")
                            .short('i')
                            .long("input")
                            .value_name("FILE")
                            .help("Specify the input YAML configuration file"),
                    )
                    .arg(
                        Arg::new("overwrite")
                            .short('o')
                            .long("overwrite")
                            .help("Overwrite existing files if specified")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .short('d')
                            .long("dry-run")
                            .help("Dry run")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .get_matches_from(args);
        if let Some(sub_m) = matches.subcommand_matches("apply") {
            let result = run_apply(sub_m);
            assert!(result.is_ok());

            // Verify that the tasks were created correctly.
            let tasks = traverse_structure(Path::new("."), &read_config(&config_path).unwrap());
            assert_eq!(tasks.len(), 2); // One directory and one file task.
            assert!(matches!(tasks[0], crate::tasks::Task::Dir(_)));
            assert!(matches!(tasks[1], crate::tasks::Task::File(_, _)));
        } else {
            panic!("Apply subcommand not found");
        }
        env::set_current_dir(orig_dir).unwrap();
    }

    #[test]
    fn test_parse_arguments_with_overwrite_apply() {
        let args = vec![
            "skeletor",
            "apply",
            "--input",
            "structure.yaml",
            "--overwrite",
        ];
        let matches = Command::new("Skeletor")
            .subcommand(
                Command::new("apply")
                    .arg(
                        Arg::new("input")
                            .short('i')
                            .long("input")
                            .value_name("FILE")
                            .help("Specify the input YAML configuration file"),
                    )
                    .arg(
                        Arg::new("overwrite")
                            .short('o')
                            .long("overwrite")
                            .help("Overwrite existing files if specified")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .short('d')
                            .long("dry-run")
                            .help("Dry run")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            assert_eq!(sub_m.get_one::<String>("input").unwrap(), "structure.yaml");
            assert_eq!(*sub_m.get_one::<bool>("overwrite").unwrap(), true);
            // By default, dry_run should be false.
            assert_eq!(*sub_m.get_one::<bool>("dry_run").unwrap_or(&false), false);
        } else {
            panic!("Apply subcommand not found");
        }
    }
}
