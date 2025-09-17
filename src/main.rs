mod apply;
mod config;
mod errors;
mod info;
mod snapshot;
mod tasks;

use crate::apply::run_apply;
use crate::errors::SkeletorError;
use crate::info::run_info;
use crate::snapshot::run_snapshot;
use clap::{Arg, ArgAction, Command};
use termcolor::{StandardStream, ColorChoice, Color, ColorSpec, WriteColor};
use std::io::Write;

/// Print a colored error message
fn print_error(message: &str) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true));
    let _ = write!(stderr, "error: ");
    let _ = stderr.reset();
    eprintln!("{}", message);
}

/// Build the CLI interface with three subcommands: `apply`, `snapshot` and `info`
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Skeletor")
        .version("0.3.0")
        .author("Jason Joseph Nathan")
        .about("A blazing-fast Rust scaffolding tool with snapshot capabilities.\n\nSkeletor helps you create project templates and scaffold new projects from YAML configurations.\nYou can capture existing folder structures as templates and apply them to create new projects.\n\nCommon workflow:\n  1. skeletor snapshot my-project -o template.yml  # Capture existing project\n  2. skeletor apply template.yml                   # Apply template elsewhere")
        .subcommand_required(true)
        .subcommand(
            Command::new("apply")
                .about("Creates files and directories based on a YAML configuration\n\nEXAMPLES:\n  skeletor apply                           # Use .skeletorrc config\n  skeletor apply my-template.yml           # Use custom config\n  skeletor apply --dry-run                 # Preview changes (summary)\n  skeletor apply --dry-run --verbose       # Preview changes (full listing)")
                .arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("YAML configuration file (defaults to .skeletorrc)")
                        .index(1),
                )
                .arg(
                    Arg::new("overwrite")
                        .short('o')
                        .long("overwrite")
                        .help("Overwrite existing files if they already exist")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry_run")
                        .short('d')
                        .long("dry-run")
                        .help("Preview changes without creating files - shows clean summary by default")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show full detailed operation listing during dry-run (useful for debugging)")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Creates a .skeletorrc snapshot from an existing folder\n\nEXAMPLES:\n  skeletor snapshot my-project              # Print YAML to stdout\n  skeletor snapshot my-project -o config.yml # Save to file\n  skeletor snapshot src/ -i \"*.log\" -i target/ # Ignore build artifacts\n  skeletor snapshot --dry-run my-project    # Preview snapshot (summary)\n  skeletor snapshot --dry-run --verbose my-project # Preview with details")
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
                        .help("Save snapshot YAML to a file (prints to stdout if omitted)"),
                )
                .arg(
                    Arg::new("include_contents")
                        .long("include-contents")
                        .help("Include file contents for text files (binary files will be empty)")
                        .action(ArgAction::SetTrue)
                        .default_value("true"),
                )
                .arg(
                    Arg::new("ignore")
                        .short('i')
                        .long("ignore")
                        .value_name("PATTERN_OR_FILE")
                        .help("Exclude files from snapshot (can be used multiple times)\n  • Glob patterns: \"*.log\", \"target/*\", \"node_modules/\"\n  • Ignore files: \".gitignore\", \".dockerignore\"")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed ignore pattern matching and file processing info")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry_run")
                        .short('d')
                        .long("dry-run")
                        .help("Preview snapshot without creating files - shows clean summary by default")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("note")
                        .short('n')
                        .long("note")
                        .value_name("NOTE")
                        .help("Attach a user-defined note to the snapshot"),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("Displays metadata from a .skeletorrc file\n\nEXAMPLES:\n  skeletor info                             # Show info for .skeletorrc\n  skeletor info my-template.yml             # Show info for custom file")
                .arg(
                    Arg::new("config")
                        .value_name("CONFIG_FILE")
                        .help("YAML configuration file to inspect (defaults to .skeletorrc)")
                        .index(1),
                ),
        )
        .get_matches()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = parse_arguments();

    if let Err(e) = run_command(&matches) {
        print_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn run_command(matches: &clap::ArgMatches) -> Result<(), SkeletorError> {
    match matches.subcommand() {
        Some(("apply", sub_m)) => run_apply(sub_m)?,
        Some(("snapshot", sub_m)) => run_snapshot(sub_m)?,
        Some(("info", sub_m)) => run_info(sub_m)?,
        _ => unreachable!("A subcommand is required"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn test_parse_arguments_apply() {
        let args = vec!["skeletor", "apply", "--input", "config.yaml"];
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
            assert_eq!(sub_m.get_one::<String>("input").unwrap(), "config.yaml");
        } else {
            panic!("Apply subcommand not found");
        }
    }

    #[test]
    fn test_parse_arguments_snapshot() {
        let args = vec!["skeletor", "snapshot", "source_folder"];
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
            assert_eq!(sub_m.get_one::<String>("source").unwrap(), "source_folder");
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_parse_arguments_info() {
        let args = vec!["skeletor", "info", "--input", "config.yaml"];
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
            assert_eq!(sub_m.get_one::<String>("input").unwrap(), "config.yaml");
        } else {
            panic!("Info subcommand not found");
        }
    }
}
