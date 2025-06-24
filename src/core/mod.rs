//! Core module for VenvCleaner functionality
//!
//! This module contains the main VenvCleaner struct and core functionality
//! for finding, analyzing, and managing Python virtual environment folders.

use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use walkdir::WalkDir;
use chrono::{DateTime, Local};
use thiserror::Error;
use log::{debug, info, warn};

pub mod venv_info;
pub mod file_utils;

pub use venv_info::VenvInfo;
pub use file_utils::FileUtils;

/// Custom error types for VenvCleaner operations
#[derive(Error, Debug, Clone)]
pub enum VenvCleanerError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Path error: {path} - {message}")]
    PathError { path: String, message: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Operation cancelled by user")]
    OperationCancelled,

    #[error("No .venv folders found in the specified directory")]
    NoVenvFound,

    #[error("Multiple errors occurred: {0:?}")]
    MultipleErrors(Vec<VenvCleanerError>),
}

impl From<std::io::Error> for VenvCleanerError {
    fn from(error: std::io::Error) -> Self {
        VenvCleanerError::Io(error.to_string())
    }
}

/// Result type alias for VenvCleaner operations
pub type Result<T> = std::result::Result<T, VenvCleanerError>;

/// Main VenvCleaner struct that handles all core operations
pub struct VenvCleaner {
    /// The base directory to search from
    base_directory: PathBuf,
    /// Whether to search recursively
    recursive: bool,
    /// Whether to perform operations without confirmation
    force_mode: bool,
    /// Whether this is a dry run (no actual changes)
    dry_run: bool,
    /// Verbosity level (0 = quiet, 1 = normal, 2+ = verbose)
    verbosity: u8,
}

impl VenvCleaner {
    /// Create a new VenvCleaner instance
    ///
    /// # Arguments
    /// * `base_directory` - The directory to start searching from
    /// * `recursive` - Whether to search subdirectories recursively
    /// * `force_mode` - Whether to skip confirmation prompts
    /// * `dry_run` - Whether to perform a dry run without making changes
    /// * `verbosity` - Verbosity level for output
    ///
    /// # Returns
    /// A new VenvCleaner instance
    pub fn new(
        base_directory: PathBuf,
        recursive: bool,
        force_mode: bool,
        dry_run: bool,
        verbosity: u8,
    ) -> Self {
        Self {
            base_directory,
            recursive,
            force_mode,
            dry_run,
            verbosity,
        }
    }

    /// Find all .venv directories in the specified path
    ///
    /// # Returns
    /// A vector of VenvInfo structs containing information about found .venv directories
    pub fn find_venv_directories(&self) -> Result<Vec<VenvInfo>> {
        info!("Searching for .venv directories in: {}", self.base_directory.display());

        let mut venv_dirs = Vec::new();
        let mut errors = Vec::new();

        if self.recursive {
            // Recursive search using walkdir
            for entry in WalkDir::new(&self.base_directory)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_dir() && entry.file_name() == ".venv" {
                    match self.analyze_venv_directory(entry.path()) {
                        Ok(venv_info) => {
                            debug!("Found .venv at: {}", entry.path().display());
                            venv_dirs.push(venv_info);
                        }
                        Err(e) => {
                            warn!("Error analyzing .venv at {}: {}", entry.path().display(), e);
                            errors.push(e);
                        }
                    }
                }
            }
        } else {
            // Non-recursive search - only check the base directory
            let venv_path = self.base_directory.join(".venv");
            if venv_path.exists() && venv_path.is_dir() {
                match self.analyze_venv_directory(&venv_path) {
                    Ok(venv_info) => {
                        debug!("Found .venv at: {}", venv_path.display());
                        venv_dirs.push(venv_info);
                    }
                    Err(e) => {
                        warn!("Error analyzing .venv at {}: {}", venv_path.display(), e);
                        errors.push(e);
                    }
                }
            }
        }

        if venv_dirs.is_empty() && errors.is_empty() {
            return Err(VenvCleanerError::NoVenvFound);
        }

        if !errors.is_empty() && self.verbosity > 0 {
            warn!("Encountered {} errors while searching", errors.len());
        }

        Ok(venv_dirs)
    }

    /// Analyze a single .venv directory and create a VenvInfo struct
    ///
    /// # Arguments
    /// * `path` - Path to the .venv directory
    ///
    /// # Returns
    /// A VenvInfo struct with directory information
    fn analyze_venv_directory(&self, path: &Path) -> Result<VenvInfo> {
        let metadata = fs::metadata(path)?;

        // Get creation time
        let created = metadata.created()
            .unwrap_or_else(|_| SystemTime::now());

        // Get last modified time
        let modified = metadata.modified()
            .unwrap_or_else(|_| SystemTime::now());

        // Calculate directory size
        let size = FileUtils::calculate_directory_size(path)?;

        // Convert system times to DateTime
        let created_dt: DateTime<Local> = created.into();
        let modified_dt: DateTime<Local> = modified.into();

        Ok(VenvInfo::new(
            path.to_path_buf(),
            size,
            created_dt,
            modified_dt,
        ))
    }

    /// Delete a .venv directory
    ///
    /// # Arguments
    /// * `venv_info` - Information about the .venv directory to delete
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn delete_venv_directory(&self, venv_info: &VenvInfo) -> Result<()> {
        if self.dry_run {
            println!("DRY RUN: Would delete: {}", venv_info.path().display());
            return Ok(());
        }

        info!("Deleting .venv directory: {}", venv_info.path().display());

        // Check if we have permission to delete
        if !FileUtils::can_delete_directory(venv_info.path())? {
            return Err(VenvCleanerError::PermissionDenied {
                path: venv_info.path().display().to_string(),
            });
        }

        // Perform the deletion
        fs::remove_dir_all(venv_info.path())?;

        info!("Successfully deleted: {}", venv_info.path().display());
        Ok(())
    }

    /// Get the base directory being searched
    pub fn base_directory(&self) -> &Path {
        &self.base_directory
    }

    /// Check if recursive search is enabled
    pub fn is_recursive(&self) -> bool {
        self.recursive
    }

    /// Check if force mode is enabled
    pub fn is_force_mode(&self) -> bool {
        self.force_mode
    }

    /// Check if this is a dry run
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Get the verbosity level
    pub fn verbosity(&self) -> u8 {
        self.verbosity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_venv_cleaner_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cleaner = VenvCleaner::new(
            temp_dir.path().to_path_buf(),
            false,
            false,
            true,
            1,
        );

        assert_eq!(cleaner.base_directory(), temp_dir.path());
        assert!(!cleaner.is_recursive());
        assert!(!cleaner.is_force_mode());
        assert!(cleaner.is_dry_run());
        assert_eq!(cleaner.verbosity(), 1);
    }

    #[test]
    fn test_find_venv_directories_empty() {
        let temp_dir = TempDir::new().unwrap();
        let cleaner = VenvCleaner::new(
            temp_dir.path().to_path_buf(),
            false,
            false,
            true,
            0,
        );

        let result = cleaner.find_venv_directories();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VenvCleanerError::NoVenvFound));
    }

    #[test]
    fn test_find_venv_directories_with_venv() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().join(".venv");
        fs::create_dir(&venv_path).unwrap();

        let cleaner = VenvCleaner::new(
            temp_dir.path().to_path_buf(),
            false,
            false,
            true,
            0,
        );

        let result = cleaner.find_venv_directories();
        assert!(result.is_ok());
        let venv_dirs = result.unwrap();
        assert_eq!(venv_dirs.len(), 1);
        assert_eq!(venv_dirs[0].path(), &venv_path);
    }
}
