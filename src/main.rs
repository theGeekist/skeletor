mod apply;
mod config;
mod errors;
mod info;
mod snapshot;
mod tasks;

#[cfg(test)]
mod test_utils;

// Re-export for tests
pub use skeletor::build_cli;

use crate::apply::run_apply;
use crate::errors::SkeletorError;
use crate::info::run_info;
use crate::snapshot::run_snapshot;
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
    skeletor::build_cli().get_matches()
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
    use crate::test_utils::helpers::*;

    #[test]
    fn test_parse_arguments_apply() {
        let args = vec!["config.yaml"];
        if let Some(sub_m) = create_apply_matches(args) {
            assert_eq!(sub_m.get_one::<String>("config").unwrap(), "config.yaml");
        } else {
            panic!("Apply subcommand not found");
        }
    }

    #[test]
    fn test_parse_arguments_snapshot() {
        let args = vec!["source_folder"];
        if let Some(sub_m) = create_snapshot_matches(args) {
            assert_eq!(sub_m.get_one::<String>("source").unwrap(), "source_folder");
        } else {
            panic!("Snapshot subcommand not found");
        }
    }

    #[test]
    fn test_parse_arguments_info() {
        let args = vec!["config.yaml"];
        if let Some(sub_m) = create_info_matches(args) {
            assert_eq!(sub_m.get_one::<String>("config").unwrap(), "config.yaml");
        } else {
            panic!("Info subcommand not found");
        }
    }
}
