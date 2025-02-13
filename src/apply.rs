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
    let dry_run = matches.get_flag("dry_run");

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
    use clap::{Arg, ArgAction, Command};

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
            assert_eq!(*sub_m.get_one::<bool>("dry_run").unwrap_or(&false), false);
        } else {
            panic!("Apply subcommand not found");
        }
    }
}
