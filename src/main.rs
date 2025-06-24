//! VenvCleaner - A multi-mode application to help manage and clean up .venv folders
//!
//! This application provides three modes of operation:
//! 1. CLI mode - Command-line interface with various flags
//! 2. TUI mode - Terminal User Interface for interactive management
//! 3. GUI mode - Graphical User Interface using Qt6
//!
//! Author: VenvCleaner Team
//! License: MIT

use clap::{Arg, Command, ArgMatches};
use std::process;
use log::{info, error};

mod cli;
mod core;
#[cfg(feature = "tui")]
mod tui;

use cli::CliMode;
use core::VenvCleanerError;
#[cfg(feature = "tui")]
use tui::TuiMode;

/// Main entry point for the VenvCleaner application
fn main() {
    // Initialize logger
    env_logger::init();

    info!("Starting VenvCleaner application");

    // Parse command line arguments
    let matches = build_cli().get_matches();

    // Execute the application based on the mode selected
    if let Err(e) = run_application(&matches) {
        error!("Application error: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Build the command line interface structure
fn build_cli() -> Command {
    Command::new("venv_cleaner")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A multi-mode application to help manage and clean up .venv folders on Mac and Linux")
        .long_about("VenvCleaner helps you find, analyze, and clean up Python virtual environment folders (.venv) on your system. It supports three modes: CLI for command-line operations, TUI for interactive terminal interface, and GUI for graphical interface.")
        .arg(
            Arg::new("directory")
                .help("Directory to search for .venv folders")
                .value_name("DIR")
                .index(1)
                .required(false)
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recursively search from the specified directory (default for TUI mode)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-recursive")
                .long("no-recursive")
                .help("Disable recursive search (TUI mode only)")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("recursive")
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Force delete without prompting")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .help("Query and display .venv folders with their sizes")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("tui")
                .long("tui")
                .help("Launch in Terminal User Interface mode")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with_all(&["gui", "query", "force"])
        )
        .arg(
            Arg::new("gui")
                .long("gui")
                .help("Launch in Graphical User Interface mode")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with_all(&["tui", "query", "force"])
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::Count)
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be deleted without actually deleting")
                .action(clap::ArgAction::SetTrue)
        )
}

/// Run the application based on the parsed command line arguments
fn run_application(matches: &ArgMatches) -> Result<(), VenvCleanerError> {
    // Determine the operating mode
    let mode = determine_mode(matches);

    info!("Operating in {:?} mode", mode);

    match mode {
        AppMode::Cli => {
            let cli_mode = CliMode::new(matches)?;
            cli_mode.execute()
        }
        AppMode::Tui => {
            #[cfg(feature = "tui")]
            {
                // Extract TUI-specific arguments
                let base_directory = if let Some(dir) = matches.get_one::<String>("directory") {
                    std::path::PathBuf::from(dir)
                } else {
                    std::env::current_dir()?
                };

                // TUI mode defaults to recursive unless explicitly disabled
                let recursive = if matches.get_flag("no-recursive") {
                    false
                } else if matches.get_flag("recursive") {
                    true
                } else {
                    true // Default to recursive for TUI mode
                };
                let verbosity = matches.get_count("verbose");

                // Create and run TUI mode
                let mut tui_mode = TuiMode::new(base_directory, recursive, verbosity)?;
                tui_mode.run()
            }
            #[cfg(not(feature = "tui"))]
            {
                eprintln!("TUI mode is not available in this build. Please rebuild with --features tui");
                Err(VenvCleanerError::FeatureNotAvailable("TUI".to_string()))
            }
        }
        AppMode::Gui => {
            #[cfg(feature = "gui")]
            {
                // GUI mode implementation will be added in future iterations
                println!("GUI mode is not yet implemented. Please use CLI mode for now.");
                println!("Use 'venv_cleaner --help' to see available CLI options.");
                Ok(())
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("GUI mode is not available in this build. Please rebuild with --features gui");
                Err(VenvCleanerError::FeatureNotAvailable("GUI".to_string()))
            }
        }
    }
}

/// Determine the application mode based on command line arguments
fn determine_mode(matches: &ArgMatches) -> AppMode {
    if matches.get_flag("tui") {
        AppMode::Tui
    } else if matches.get_flag("gui") {
        AppMode::Gui
    } else {
        AppMode::Cli
    }
}

/// Application operating modes
#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    /// Command Line Interface mode
    Cli,
    /// Terminal User Interface mode
    Tui,
    /// Graphical User Interface mode
    Gui,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_building() {
        let cmd = build_cli();
        assert_eq!(cmd.get_name(), "venv_cleaner");
    }

    #[test]
    fn test_mode_determination() {
        let matches = build_cli().try_get_matches_from(&["venv_cleaner"]).unwrap();
        assert_eq!(determine_mode(&matches), AppMode::Cli);
    }

    #[test]
    fn test_tui_mode_determination() {
        let matches = build_cli().try_get_matches_from(&["venv_cleaner", "--tui"]).unwrap();
        assert_eq!(determine_mode(&matches), AppMode::Tui);
    }

    #[test]
    fn test_gui_mode_determination() {
        let matches = build_cli().try_get_matches_from(&["venv_cleaner", "--gui"]).unwrap();
        assert_eq!(determine_mode(&matches), AppMode::Gui);
    }
}
