//! CLI module for VenvCleaner
//!
//! This module handles command-line interface operations for the VenvCleaner application.
//! It provides functionality for interactive and non-interactive .venv directory management.

use clap::ArgMatches;
use std::path::PathBuf;
use std::io::{self, Write};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;

use crate::core::{VenvCleaner, VenvCleanerError, VenvInfo, Result};

/// CLI mode handler for VenvCleaner
pub struct CliMode {
    /// The core VenvCleaner instance
    cleaner: VenvCleaner,
    /// Whether to query and display information only
    query_mode: bool,
    /// Whether to show progress bars
    show_progress: bool,
}

impl CliMode {
    /// Create a new CliMode instance from command line arguments
    ///
    /// # Arguments
    /// * `matches` - Parsed command line arguments
    ///
    /// # Returns
    /// A new CliMode instance or an error
    pub fn new(matches: &ArgMatches) -> Result<Self> {
        // Determine the base directory
        let base_directory = if let Some(dir) = matches.get_one::<String>("directory") {
            PathBuf::from(dir)
        } else {
            std::env::current_dir()?
        };

        // Validate that the directory exists
        if !base_directory.exists() {
            return Err(VenvCleanerError::PathError {
                path: base_directory.display().to_string(),
                message: "Directory does not exist".to_string(),
            });
        }

        if !base_directory.is_dir() {
            return Err(VenvCleanerError::PathError {
                path: base_directory.display().to_string(),
                message: "Path is not a directory".to_string(),
            });
        }

        // Extract other options
        let recursive = matches.get_flag("recursive");
        let force_mode = matches.get_flag("force");
        let dry_run = matches.get_flag("dry-run");
        let query_mode = matches.get_flag("query");
        let verbosity = matches.get_count("verbose");

        // Create the VenvCleaner instance
        let cleaner = VenvCleaner::new(
            base_directory,
            recursive,
            force_mode,
            dry_run,
            verbosity,
        );

        Ok(Self {
            cleaner,
            query_mode,
            show_progress: verbosity == 0, // Show progress only when not in verbose mode
        })
    }

    /// Execute the CLI mode operations
    pub fn execute(&self) -> Result<()> {
        info!("Executing CLI mode");

        // Print initial information
        self.print_header();

        // Find .venv directories
        let venv_dirs = self.find_venv_directories()?;

        if self.query_mode {
            self.handle_query_mode(&venv_dirs)
        } else {
            self.handle_cleanup_mode(&venv_dirs)
        }
    }

    /// Find .venv directories with optional progress indication
    fn find_venv_directories(&self) -> Result<Vec<VenvInfo>> {
        let progress = if self.show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner())
            );
            pb.set_message("Searching for .venv directories...");
            Some(pb)
        } else {
            None
        };

        let result = self.cleaner.find_venv_directories();

        if let Some(pb) = progress {
            pb.finish_with_message("Search completed");
        }

        result
    }

    /// Handle query mode (list .venv directories with information)
    fn handle_query_mode(&self, venv_dirs: &[VenvInfo]) -> Result<()> {
        println!("\n{}", "Found .venv directories:".bold().green());
        println!("{}", "=".repeat(80).dimmed());

        if venv_dirs.is_empty() {
            println!("{}", "No .venv directories found.".yellow());
            return Ok(());
        }

        // Sort by size (largest first) for query mode
        let mut sorted_dirs = venv_dirs.to_vec();
        sorted_dirs.sort_by(|a, b| a.compare_by_size(b));

        // Calculate totals
        let total_size: u64 = venv_dirs.iter().map(|v| v.size_bytes()).sum();
        let total_count = venv_dirs.len();

        // Print header
        println!(
            "{:<60} {:<12} {:<20} {:<20}",
            "Location".bold(),
            "Size".bold(),
            "Created".bold(),
            "Last Used".bold()
        );
        println!("{}", "-".repeat(120).dimmed());

        // Print each .venv directory
        for venv_info in &sorted_dirs {
            let location = self.format_location_for_display(&venv_info.location(), 58);
            let size = if venv_info.size_bytes() > 1024 * 1024 * 1024 {
                venv_info.size_formatted().red().to_string()
            } else if venv_info.size_bytes() > 100 * 1024 * 1024 {
                venv_info.size_formatted().yellow().to_string()
            } else {
                venv_info.size_formatted().normal().to_string()
            };

            let age_color = if venv_info.is_recently_used() {
                "green"
            } else if venv_info.is_old() {
                "red"
            } else {
                "normal"
            };

            let last_used = match age_color {
                "green" => venv_info.last_modified_formatted().green().to_string(),
                "red" => venv_info.last_modified_formatted().red().to_string(),
                _ => venv_info.last_modified_formatted(),
            };

            println!(
                "{:<60} {:<12} {:<20} {:<20}",
                location,
                size,
                venv_info.created_formatted().dimmed(),
                last_used
            );
        }

        // Print summary
        println!("{}", "-".repeat(120).dimmed());
        println!(
            "\n{} {} .venv directories found, total size: {}",
            "Summary:".bold(),
            total_count.to_string().cyan(),
            Self::format_size(total_size).cyan()
        );

        // Show recommendations
        self.print_recommendations(&sorted_dirs);

        Ok(())
    }

    /// Handle cleanup mode (delete .venv directories)
    fn handle_cleanup_mode(&self, venv_dirs: &[VenvInfo]) -> Result<()> {
        if venv_dirs.is_empty() {
            println!("{}", "No .venv directories found.".yellow());
            return Ok(());
        }

        println!("\n{} {} .venv directories found:",
                "Found".green(),
                venv_dirs.len().to_string().cyan());

        let mut deleted_count = 0;
        let mut total_freed = 0u64;
        let mut errors = Vec::new();

        for venv_info in venv_dirs {
            match self.process_venv_directory(venv_info) {
                Ok(deleted) => {
                    if deleted {
                        deleted_count += 1;
                        total_freed += venv_info.size_bytes();
                    }
                }
                Err(e) => {
                    errors.push((venv_info.path().display().to_string(), e));
                }
            }
        }

        // Print summary
        self.print_cleanup_summary(deleted_count, total_freed, &errors);

        Ok(())
    }

    /// Process a single .venv directory (prompt and potentially delete)
    fn process_venv_directory(&self, venv_info: &VenvInfo) -> Result<bool> {
        let location = venv_info.location();
        let size = venv_info.size_formatted();
        let age_days = venv_info.age_in_days();

        // Show information about this .venv
        println!("\n{}", "─".repeat(60).dimmed());
        println!("📁 {}", location.cyan());
        println!("📏 Size: {}", size);
        println!("📅 Last used: {} ({} days ago)",
                venv_info.last_modified_formatted().dimmed(),
                age_days);

        // Add age-based coloring and warnings
        if venv_info.is_old() {
            println!("⚠️  {}", "This .venv hasn't been used in over 90 days".yellow());
        } else if venv_info.is_recently_used() {
            println!("✨ {}", "This .venv was used recently".green());
        }

        // In force mode, delete without asking
        if self.cleaner.is_force_mode() {
            println!("🗑️  {}", "Force mode: deleting...".red());
            self.cleaner.delete_venv_directory(venv_info)?;
            println!("✅ {}", "Deleted successfully".green());
            return Ok(true);
        }

        // Ask user for confirmation
        print!("\n{} (y/N): ", "Delete this .venv directory?".bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let should_delete = input.trim().to_lowercase();
        if should_delete == "y" || should_delete == "yes" {
            println!("🗑️  {}", "Deleting...".yellow());
            self.cleaner.delete_venv_directory(venv_info)?;
            println!("✅ {}", "Deleted successfully".green());
            Ok(true)
        } else {
            println!("⏭️  {}", "Skipped".dimmed());
            Ok(false)
        }
    }

    /// Print the application header
    fn print_header(&self) {
        println!("{}", "VenvCleaner".bold().green());
        println!("{}", "Python Virtual Environment Cleanup Tool".dimmed());
        println!();

        // Show current configuration
        println!("🔍 Searching in: {}", self.cleaner.base_directory().display().to_string().cyan());

        if self.cleaner.is_recursive() {
            println!("📂 Mode: {}", "Recursive search".yellow());
        } else {
            println!("📂 Mode: {}", "Current directory only".normal());
        }

        if self.cleaner.is_dry_run() {
            println!("🧪 {}", "DRY RUN MODE - No files will be deleted".yellow().bold());
        }

        if self.cleaner.is_force_mode() {
            println!("⚡ {}", "FORCE MODE - Will delete without prompting".red().bold());
        }

        if self.query_mode {
            println!("📊 {}", "QUERY MODE - Will only display information".blue().bold());
        }
    }

    /// Print recommendations based on found .venv directories
    fn print_recommendations(&self, venv_dirs: &[VenvInfo]) {
        let old_dirs: Vec<_> = venv_dirs.iter().filter(|v| v.is_old()).collect();
        let large_dirs: Vec<_> = venv_dirs.iter().filter(|v| v.size_bytes() > 500 * 1024 * 1024).collect();

        if !old_dirs.is_empty() || !large_dirs.is_empty() {
            println!("\n{}", "Recommendations:".bold().yellow());
        }

        if !old_dirs.is_empty() {
            println!("🧹 {} old .venv directories (>90 days) could be cleaned up",
                    old_dirs.len().to_string().red());
        }

        if !large_dirs.is_empty() {
            println!("📦 {} large .venv directories (>500MB) are taking significant space",
                    large_dirs.len().to_string().yellow());
        }

        if !old_dirs.is_empty() {
            println!("\n💡 Consider running: {} to clean up old directories",
                    "venv_cleaner -r -f".green());
        }
    }

    /// Print cleanup operation summary
    fn print_cleanup_summary(&self, deleted_count: usize, total_freed: u64, errors: &[(String, VenvCleanerError)]) {
        println!("\n{}", "=".repeat(60).green());
        println!("{}", "Cleanup Summary".bold().green());
        println!("{}", "=".repeat(60).green());

        if self.cleaner.is_dry_run() {
            println!("🧪 {} directories would be deleted", deleted_count.to_string().cyan());
            println!("💾 {} would be freed", Self::format_size(total_freed).cyan());
        } else {
            println!("✅ {} directories deleted", deleted_count.to_string().green());
            println!("💾 {} freed", Self::format_size(total_freed).green());
        }

        if !errors.is_empty() {
            println!("❌ {} errors occurred:", errors.len().to_string().red());
            for (path, error) in errors {
                println!("   • {}: {}", path.red(), error.to_string().dimmed());
            }
        }

        if deleted_count > 0 && !self.cleaner.is_dry_run() {
            println!("\n🎉 {}", "Cleanup completed successfully!".green().bold());
        }
    }

    /// Format a location string for display, truncating if necessary
    fn format_location_for_display(&self, location: &str, max_width: usize) -> String {
        if location.len() <= max_width {
            location.to_string()
        } else {
            format!("...{}", &location[location.len() - max_width + 3..])
        }
    }

    /// Format a size in bytes to a human-readable string
    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;
    use tempfile::TempDir;

    fn create_test_command() -> Command {
        Command::new("test")
            .arg(clap::Arg::new("directory").index(1))
            .arg(clap::Arg::new("recursive").short('r').action(clap::ArgAction::SetTrue))
            .arg(clap::Arg::new("force").short('f').action(clap::ArgAction::SetTrue))
            .arg(clap::Arg::new("query").short('q').action(clap::ArgAction::SetTrue))
            .arg(clap::Arg::new("dry-run").long("dry-run").action(clap::ArgAction::SetTrue))
            .arg(clap::Arg::new("verbose").short('v').action(clap::ArgAction::Count))
    }

    #[test]
    fn test_cli_mode_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(&[
            "test",
            temp_dir.path().to_str().unwrap()
        ]).unwrap();

        let cli_mode = CliMode::new(&matches);
        assert!(cli_mode.is_ok());
    }

    #[test]
    fn test_cli_mode_nonexistent_directory() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(&[
            "test",
            "/nonexistent/directory"
        ]).unwrap();

        let cli_mode = CliMode::new(&matches);
        assert!(cli_mode.is_err());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(CliMode::format_size(500), "500 bytes");
        assert_eq!(CliMode::format_size(1536), "1.50 KB");
        assert_eq!(CliMode::format_size(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(CliMode::format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_location_for_display() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(&[
            "test",
            temp_dir.path().to_str().unwrap()
        ]).unwrap();

        let cli_mode = CliMode::new(&matches).unwrap();

        let short_path = "/short/path";
        let formatted = cli_mode.format_location_for_display(short_path, 50);
        assert_eq!(formatted, short_path);

        let long_path = "/very/long/path/that/exceeds/maximum/width/allowed/for/display";
        let formatted = cli_mode.format_location_for_display(long_path, 20);
        assert!(formatted.len() <= 20);
        assert!(formatted.starts_with("..."));
    }
}
