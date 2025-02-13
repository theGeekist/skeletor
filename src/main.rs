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

/// Build the CLI interface with three subcommands: `apply`, `snapshot` and `info`
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Skeletor")
        .version("2.2.11"
        .author("Jason Joseph Nathan")
        .about("A super optimised Rust scaffolding tool with snapshot annotations")
        .subcommand_required(true)
        .subcommand(
            Command::new("apply")
                .about("Applies a YAML configuration to generate files and directories")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the input YAML configuration file (defaults to .skeletorrc)"),
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
                        .help("Perform a trial run with no changes made")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Generates a .skeletorrc snapshot from an existing folder")
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
        .subcommand(
            Command::new("info")
                .about("Prints annotation information from a .skeletorrc file")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Specify the YAML configuration file (defaults to .skeletorrc)"),
                ),
        )
        .get_matches()
}

fn main() -> Result<(), SkeletorError> {
    env_logger::init();

    let matches = parse_arguments();

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
