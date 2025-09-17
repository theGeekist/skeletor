use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::tasks::{create_files_and_directories, task_path, traverse_structure, Task};
use clap::ArgMatches;
use log::info;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Helper function to print colored timing information
fn print_colored_apply_duration(duration: std::time::Duration) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    print!("Successfully generated files and directories in ");
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
    let _ = write!(stdout, "{:.2}ms", duration.as_micros() as f64 / 1000.0);
    let _ = stdout.reset();
    println!();
}

/// Runs the apply subcommand: reads the YAML config and creates files/directories.
/// In dry-run mode, the tasks are printed without performing any filesystem changes.
pub fn run_apply(matches: &ArgMatches) -> Result<(), SkeletorError> {
    // Use default_file_path so that .skeletorrc is used by default if no config is provided.
    let input_path = default_file_path(matches.get_one::<String>("config"));
    let overwrite = *matches.get_one::<bool>("overwrite").unwrap_or(&false);
    let dry_run = matches.get_flag("dry_run");
    let verbose = matches.get_flag("verbose");

    info!("Reading input file: {:?}", input_path);
    info!("Overwrite flag: {:?}", overwrite);

    // Add debug output to print the input path
    println!("Input path: {:?}", input_path);

    let config = read_config(&input_path)?;

    if config.is_null() {
        return Err(SkeletorError::Config(
            "'directories' key is required in the YAML file".into(),
        ));
    }

    let start_time = Instant::now();

    let tasks = traverse_structure(Path::new("."), &config);

    if dry_run {
        // Count files and directories
        let mut file_count = 0;
        let mut dir_count = 0;
        for task in tasks.iter() {
            match task {
                crate::tasks::Task::File { .. } => file_count += 1,
                crate::tasks::Task::Dir { .. } => dir_count += 1,
            }
        }

        if verbose {
            // Verbose mode: Full listing for debugging (prettier format)
            println!("Dry run enabled. Detailed operation listing:");
            println!("════════════════════════════════════════");
            
            let mut dir_counter = 0;
            let mut file_counter = 0;
            
            for task in tasks.iter() {
                match task {
                    crate::tasks::Task::Dir { .. } => {
                        dir_counter += 1;
                        print!("📁 [{:4}] ", dir_counter);
                        println!("{}", task_path(task));
                    }
                    crate::tasks::Task::File { .. } => {
                        file_counter += 1;
                        print!("📄 [{:4}] ", file_counter);
                        println!("{}", task_path(task));
                    }
                }
            }
            
            println!("════════════════════════════════════════");
            println!("Summary: {} directories, {} files ({} total operations)", dir_count, file_count, tasks.len());
        } else {
            // Default mode: Clean summary
            println!("Dry run enabled. Summary of planned operations:");
            println!("  • {} files to be created", file_count);
            println!("  • {} directories to be created", dir_count);
            println!("  • Total: {} operations", tasks.len());
            
            // Show a sample of what would be created (first 5 items)
            if !tasks.is_empty() {
                println!("\nSample of operations:");
                for (i, task) in tasks.iter().take(5).enumerate() {
                    match task {
                        Task::Dir(_) => {
                            print!("  {}. 📁 ", i + 1);
                            println!("{}", task_path(task));
                        },
                        Task::File(_, _) => {
                            print!("  {}. 📄 ", i + 1);
                            println!("{}", task_path(task));
                        },
                    }
                }
                if tasks.len() > 5 {
                    println!("  ... and {} more operations", tasks.len() - 5);
                }
                println!("\ntip: Use --verbose to see the complete operation list");
            }
        }
        
        println!("\nDry run complete. No changes were made.");
    } else {
        create_files_and_directories(&tasks, overwrite)?;
        let duration = start_time.elapsed();
        print_colored_apply_duration(duration);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    fn create_test_command() -> Command {
        Command::new("Skeletor")
            .subcommand(
                Command::new("apply")
                    .arg(
                        Arg::new("config")
                            .value_name("CONFIG_FILE")
                            .help("Specify the YAML configuration file")
                            .index(1),
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
                    )
                    .arg(
                        Arg::new("verbose")
                            .short('v')
                            .long("verbose")
                            .help("Verbose output")
                            .action(ArgAction::SetTrue),
                    ),
            )
    }

    #[test]
    fn test_parse_arguments_with_overwrite_apply() {
        let args = vec![
            "skeletor",
            "apply",
            "structure.yaml",
            "--overwrite",
        ];
        let matches = create_test_command().get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            assert_eq!(sub_m.get_one::<String>("config").unwrap(), "structure.yaml");
            assert!(*sub_m.get_one::<bool>("overwrite").unwrap());
            assert!(!(*sub_m.get_one::<bool>("dry_run").unwrap_or(&false)));
        } else {
            panic!("Apply subcommand not found");
        }
    }

    #[test]
    fn test_apply_with_missing_config_file() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let non_existent_file = temp_dir.path().join("missing.yml");
        
        let args = vec![
            "skeletor",
            "apply",
            non_existent_file.to_str().unwrap(),
        ];
        
        let matches = create_test_command().get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            let result = crate::apply::run_apply(sub_m);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_apply_with_dry_run() {
        use tempfile::tempdir;
        use std::fs;
        
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("test.yml");
        
        // Create a valid YAML config
        let config_content = r#"
directories:
  test_dir:
    test_file.txt: "Hello, World!"
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let args = vec![
            "skeletor",
            "apply",
            config_file.to_str().unwrap(),
            "--dry-run",
        ];
        
        let matches = create_test_command().get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            let result = crate::apply::run_apply(sub_m);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_apply_with_invalid_yaml() {
        use tempfile::tempdir;
        use std::fs;
        
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("invalid.yml");
        
        // Create invalid YAML
        fs::write(&config_file, "invalid: yaml: content: [").unwrap();
        
        let args = vec![
            "skeletor",
            "apply",
            config_file.to_str().unwrap(),
        ];
        
        let matches = create_test_command().get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            let result = crate::apply::run_apply(sub_m);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_apply_without_directories_key() {
        use tempfile::tempdir;
        use std::fs;
        
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("no_dirs.yml");
        
        // Create YAML without directories key
        fs::write(&config_file, "other_key: value").unwrap();
        
        let args = vec![
            "skeletor",
            "apply",
            config_file.to_str().unwrap(),
        ];
        
        let matches = create_test_command().get_matches_from(args);

        if let Some(sub_m) = matches.subcommand_matches("apply") {
            let result = crate::apply::run_apply(sub_m);
            assert!(result.is_err());
        }
    }
}
