use crate::config::{default_file_path, read_config};
use crate::errors::SkeletorError;
use crate::tasks::{create_files_and_directories, task_path, traverse_structure, Task};
use clap::ArgMatches;
use log::info;
use std::path::Path;
use std::time::Instant;
use termcolor::{StandardStream, ColorChoice, Color, ColorSpec, WriteColor};
use std::io::Write;

/// Helper function to print colored timing information with standardized formatting
fn print_colored_duration(prefix: &str, duration: std::time::Duration) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    print!("{}", prefix);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
    let _ = write!(stdout, "{:.2}ms", duration.as_micros() as f64 / 1000.0);
    let _ = stdout.reset();
    println!();
}

/// Counts files and directories in a task list
fn count_tasks(tasks: &[Task]) -> (usize, usize) {
    let mut file_count = 0;
    let mut dir_count = 0;
    for task in tasks.iter() {
        match task {
            Task::File { .. } => file_count += 1,
            Task::Dir { .. } => dir_count += 1,
        }
    }
    (file_count, dir_count)
}

/// Handles dry-run output display with verbose and summary modes
fn display_dry_run_output(tasks: &[Task], verbose: bool) {
    let (file_count, dir_count) = count_tasks(tasks);
    
    if verbose {
        // Verbose mode: Full listing for debugging (prettier format)
        println!("Dry run enabled. Detailed operation listing:");
        println!("════════════════════════════════════════");
        
        let mut dir_counter = 0;
        let mut file_counter = 0;
        
        for task in tasks.iter() {
            match task {
                Task::Dir { .. } => {
                    dir_counter += 1;
                    print!("📁 [{:4}] ", dir_counter);
                    println!("{}", task_path(task));
                }
                Task::File { .. } => {
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
}

/// Parses CLI arguments and extracts apply-specific configuration
struct ApplyConfig {
    pub input_path: std::path::PathBuf,
    pub overwrite: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ApplyConfig {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            input_path: default_file_path(matches.get_one::<String>("config")),
            overwrite: *matches.get_one::<bool>("overwrite").unwrap_or(&false),
            dry_run: matches.get_flag("dry_run"),
            verbose: matches.get_flag("verbose"),
        }
    }
}

/// Runs the apply subcommand: reads the YAML config and creates files/directories.
/// In dry-run mode, the tasks are printed without performing any filesystem changes.
pub fn run_apply(matches: &ArgMatches) -> Result<(), SkeletorError> {
    let config = ApplyConfig::from_matches(matches);

    info!("Reading input file: {:?}", config.input_path);
    info!("Overwrite flag: {:?}", config.overwrite);

    // Add debug output to print the input path
    println!("Input path: {:?}", config.input_path);

    let yaml_config = read_config(&config.input_path)?;

    if yaml_config.is_null() {
        return Err(SkeletorError::Config(
            "'directories' key is required in the YAML file".into(),
        ));
    }

    let start_time = Instant::now();
    let tasks = traverse_structure(Path::new("."), &yaml_config);

    if config.dry_run {
        display_dry_run_output(&tasks, config.verbose);
    } else {
        create_files_and_directories(&tasks, config.overwrite)?;
        let duration = start_time.elapsed();
        print_colored_duration("Successfully generated files and directories in ", duration);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test_utils::helpers::*;

    #[test]
    fn test_parse_arguments_with_overwrite_apply() {
        let args = vec![
            "structure.yaml",
            "--overwrite",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_eq!(sub_m.get_one::<String>("config").unwrap(), "structure.yaml");
            assert!(*sub_m.get_one::<bool>("overwrite").unwrap());
            assert!(!(*sub_m.get_one::<bool>("dry_run").unwrap_or(&false)));
        } else {
            panic!("Apply subcommand not found");
        }
    }

    #[test]
    fn test_apply_with_missing_config_file() {
        let fs = TestFileSystem::new();
        let non_existent_file = fs.path("missing.yml");
        
        let args = vec![non_existent_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_with_dry_run() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_test_config("test.yml");
        
        let args = vec![
            config_file.to_str().unwrap(),
            "--dry-run",
        ];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_succeeds(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_with_invalid_yaml() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_invalid_config("invalid.yml");
        
        let args = vec![config_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }

    #[test]
    fn test_apply_without_directories_key() {
        let fs = TestFileSystem::new();
        let config_file = fs.create_config_without_directories("no_dirs.yml");
        
        let args = vec![config_file.to_str().unwrap()];
        
        if let Some(sub_m) = create_apply_matches(args) {
            assert_command_fails(|| crate::apply::run_apply(&sub_m));
        }
    }
}
