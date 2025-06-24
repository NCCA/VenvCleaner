//! GUI module for VenvCleaner
//!
//! This module handles the Graphical User Interface for interactive .venv directory management.
//! It provides a full GUI using egui with mouse and keyboard navigation,
//! sorting options, and interactive deletion capabilities similar to the TUI version.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{self, *};
use log::{debug, info, warn};

use crate::core::{VenvCleaner, VenvCleanerError, VenvInfo, Result};

pub mod app;
pub mod components;
pub mod theme;

pub use app::GuiApp;
pub use theme::Theme;

/// Main GUI mode handler for VenvCleaner
pub struct GuiMode {
    /// The core VenvCleaner instance
    cleaner: VenvCleaner,
    /// Base directory for searching
    base_directory: PathBuf,
    /// Whether to search recursively
    recursive: bool,
    /// Verbosity level
    verbosity: u8,
}

/// Application states for the GUI
#[derive(Debug, Clone, PartialEq)]
pub enum GuiAppState {
    /// Loading .venv directories
    Loading,
    /// Browsing .venv directories
    Browsing,
    /// Confirming deletion of selected directories
    ConfirmingDeletion,
    /// Deleting directories
    Deleting,
    /// Showing error message
    Error,
    /// Application should quit
    Quit,
}

/// Sorting options for .venv directories (same as TUI)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "gui", derive(serde::Serialize, serde::Deserialize))]
pub enum GuiSortBy {
    /// Sort by path (alphabetical)
    Path,
    /// Sort by size (largest first)
    Size,
    /// Sort by creation date (newest first)
    Created,
    /// Sort by last modified date (most recent first)
    LastModified,
}

impl GuiSortBy {
    /// Get the next sort option in sequence
    pub fn next(self) -> Self {
        match self {
            GuiSortBy::Path => GuiSortBy::Size,
            GuiSortBy::Size => GuiSortBy::Created,
            GuiSortBy::Created => GuiSortBy::LastModified,
            GuiSortBy::LastModified => GuiSortBy::Path,
        }
    }

    /// Get the display name for this sort option
    pub fn display_name(self) -> &'static str {
        match self {
            GuiSortBy::Path => "Path",
            GuiSortBy::Size => "Size",
            GuiSortBy::Created => "Created",
            GuiSortBy::LastModified => "Last Used",
        }
    }
}

/// Background task events
#[derive(Debug)]
pub enum GuiEvent {
    /// .venv directories have been loaded
    VenvsLoaded(Vec<VenvInfo>),
    /// Error occurred while loading .venv directories
    LoadError(String),
    /// Deletion operation completed
    DeletionComplete(Vec<(VenvInfo, Result<()>)>),
}

impl GuiMode {
    /// Create a new GuiMode instance
    pub fn new(
        base_directory: PathBuf,
        recursive: bool,
        verbosity: u8,
    ) -> Result<Self> {
        info!("Creating GUI mode");

        // Create VenvCleaner instance
        let cleaner = VenvCleaner::new(
            base_directory.clone(),
            recursive,
            false, // force_mode = false for GUI
            false, // dry_run = false (we handle this in GUI)
            verbosity,
        );

        Ok(Self {
            cleaner,
            base_directory,
            recursive,
            verbosity,
        })
    }

    /// Run the GUI application
    pub fn run(self) -> Result<()> {
        info!("Starting GUI mode");

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([800.0, 600.0])
                .with_title("VenvCleaner - Python Virtual Environment Manager")
                .with_icon(
                    // Try to load an icon, but don't fail if we can't
                    eframe::icon_data::from_png_bytes(&[])
                        .unwrap_or_else(|_| egui::IconData { rgba: vec![], width: 0, height: 0 })
                ),
            ..Default::default()
        };

        // Create the GUI app
        let gui_app = GuiApp::new(self.cleaner, self.base_directory, self.recursive);

        // Run the application
        eframe::run_native(
            "VenvCleaner",
            options,
            Box::new(|_cc| Box::new(gui_app)),
        )
        .map_err(|e| VenvCleanerError::Io(format!("Failed to run GUI: {}", e)))?;

        Ok(())
    }
}

/// Helper functions for GUI operations
pub mod utils {
    use super::*;

    /// Format file size for display
    pub fn format_size(bytes: u64) -> String {
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

    /// Format a file path for display, truncating if necessary
    pub fn format_path_for_display(path: &str, max_length: usize) -> String {
        if path.len() <= max_length {
            path.to_string()
        } else {
            format!("...{}", &path[path.len().saturating_sub(max_length - 3)..])
        }
    }

    /// Get color for size based on magnitude
    pub fn get_size_color(bytes: u64) -> Color32 {
        const MB_100: u64 = 100 * 1024 * 1024;
        const GB_1: u64 = 1024 * 1024 * 1024;

        if bytes >= GB_1 {
            Color32::from_rgb(255, 100, 100) // Red for > 1GB
        } else if bytes >= MB_100 {
            Color32::from_rgb(255, 200, 100) // Orange for > 100MB
        } else {
            Color32::from_rgb(200, 200, 200) // Gray for smaller sizes
        }
    }

    /// Get color for age based on days
    pub fn get_age_color(days: i64) -> Color32 {
        if days <= 30 {
            Color32::from_rgb(100, 255, 100) // Green for recent
        } else if days <= 90 {
            Color32::from_rgb(255, 255, 100) // Yellow for moderate
        } else {
            Color32::from_rgb(255, 100, 100) // Red for old
        }
    }

    /// Get age indicator emoji
    pub fn get_age_indicator(days: i64) -> &'static str {
        if days <= 30 {
            "🟢" // Green circle for recent
        } else if days <= 90 {
            "🟡" // Yellow circle for moderate
        } else {
            "🔴" // Red circle for old
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_gui_sort_by_cycle() {
        assert_eq!(GuiSortBy::Path.next(), GuiSortBy::Size);
        assert_eq!(GuiSortBy::Size.next(), GuiSortBy::Created);
        assert_eq!(GuiSortBy::Created.next(), GuiSortBy::LastModified);
        assert_eq!(GuiSortBy::LastModified.next(), GuiSortBy::Path);
    }

    #[test]
    fn test_gui_sort_by_display_names() {
        assert_eq!(GuiSortBy::Path.display_name(), "Path");
        assert_eq!(GuiSortBy::Size.display_name(), "Size");
        assert_eq!(GuiSortBy::Created.display_name(), "Created");
        assert_eq!(GuiSortBy::LastModified.display_name(), "Last Used");
    }

    #[test]
    fn test_gui_mode_creation() {
        let temp_dir = TempDir::new().unwrap();
        let gui_mode = GuiMode::new(
            temp_dir.path().to_path_buf(),
            true,
            1,
        );
        assert!(gui_mode.is_ok());
    }

    #[test]
    fn test_utils_format_size() {
        use utils::format_size;

        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_utils_format_path() {
        use utils::format_path_for_display;

        assert_eq!(format_path_for_display("short", 10), "short");
        assert_eq!(format_path_for_display("very/long/path/here", 10), "...th/here");
    }
}
