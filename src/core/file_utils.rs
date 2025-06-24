//! File utilities module for VenvCleaner
//!
//! This module provides utility functions for file and directory operations,
//! including calculating directory sizes and checking permissions.

use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use walkdir::WalkDir;
use log::{debug, warn};

use super::{Result, VenvCleanerError};

/// Utility struct for file operations
pub struct FileUtils;

impl FileUtils {
    /// Calculate the total size of a directory and all its contents
    ///
    /// # Arguments
    /// * `path` - Path to the directory to calculate size for
    ///
    /// # Returns
    /// Total size in bytes, or an error if the operation fails
    pub fn calculate_directory_size(path: &Path) -> Result<u64> {
        if !path.exists() {
            return Err(VenvCleanerError::PathError {
                path: path.display().to_string(),
                message: "Directory does not exist".to_string(),
            });
        }

        if !path.is_dir() {
            return Err(VenvCleanerError::PathError {
                path: path.display().to_string(),
                message: "Path is not a directory".to_string(),
            });
        }

        let mut total_size = 0u64;
        let mut error_count = 0;

        debug!("Calculating size for directory: {}", path.display());

        // Walk through all files and directories recursively
        for entry in WalkDir::new(path).follow_links(false).into_iter() {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        match entry.metadata() {
                            Ok(metadata) => {
                                total_size = total_size.saturating_add(metadata.len());
                            }
                            Err(e) => {
                                warn!("Failed to get metadata for {}: {}", entry.path().display(), e);
                                error_count += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Error walking directory {}: {}", path.display(), e);
                    error_count += 1;
                }
            }
        }

        if error_count > 0 {
            debug!("Encountered {} errors while calculating directory size", error_count);
        }

        debug!("Total size calculated: {} bytes", total_size);
        Ok(total_size)
    }

    /// Check if a directory can be deleted (has write permissions)
    ///
    /// # Arguments
    /// * `path` - Path to the directory to check
    ///
    /// # Returns
    /// True if the directory can be deleted, false otherwise
    pub fn can_delete_directory(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // Check if we can write to the parent directory
        if let Some(parent) = path.parent() {
            match Self::check_write_permission(parent) {
                Ok(can_write) => {
                    if !can_write {
                        return Ok(false);
                    }
                }
                Err(_) => return Ok(false),
            }
        }

        // Check if we can read the directory itself
        match fs::read_dir(path) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Check write permission for a directory
    ///
    /// # Arguments
    /// * `path` - Path to check write permission for
    ///
    /// # Returns
    /// True if writable, false otherwise
    fn check_write_permission(path: &Path) -> Result<bool> {
        match fs::metadata(path) {
            Ok(metadata) => {
                // On Unix systems, we can check permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();

                    // Check owner write permission (bit 7)
                    // This is a simplified check - in reality, we'd need to check
                    // if we're the owner, in the group, or use other permissions
                    Ok((mode & 0o200) != 0)
                }

                // On other systems, assume we can write if we can read the metadata
                #[cfg(not(unix))]
                {
                    Ok(!metadata.permissions().readonly())
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Get the number of files and directories in a path
    ///
    /// # Arguments
    /// * `path` - Path to count items in
    ///
    /// # Returns
    /// Tuple of (file_count, directory_count)
    pub fn count_items(path: &Path) -> Result<(usize, usize)> {
        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in WalkDir::new(path).follow_links(false).into_iter() {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        file_count += 1;
                    } else if entry.file_type().is_dir() && entry.path() != path {
                        // Don't count the root directory itself
                        dir_count += 1;
                    }
                }
                Err(e) => {
                    warn!("Error counting items in {}: {}", path.display(), e);
                }
            }
        }

        Ok((file_count, dir_count))
    }

    /// Check if a path is a valid .venv directory
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// True if it appears to be a valid .venv directory
    pub fn is_valid_venv_directory(path: &Path) -> bool {
        if !path.is_dir() || path.file_name() != Some(std::ffi::OsStr::new(".venv")) {
            return false;
        }

        // Check for common .venv subdirectories and files
        let common_venv_items = [
            "bin",      // Unix
            "Scripts",  // Windows
            "lib",
            "include",
            "pyvenv.cfg",
        ];

        let mut found_items = 0;
        for item in &common_venv_items {
            if path.join(item).exists() {
                found_items += 1;
            }
        }

        // If we find at least 2 common items, it's likely a valid venv
        found_items >= 2
    }

    /// Format a file path for display, shortening it if necessary
    ///
    /// # Arguments
    /// * `path` - Path to format
    /// * `max_length` - Maximum length for the displayed path
    ///
    /// # Returns
    /// Formatted path string
    pub fn format_path_for_display(path: &Path, max_length: usize) -> String {
        let path_str = path.display().to_string();

        if path_str.len() <= max_length {
            return path_str;
        }

        // Try to shorten by showing only the last few components
        let components: Vec<_> = path.components().collect();
        if components.len() <= 2 {
            return path_str;
        }

        // Start with the last component and add previous ones until we exceed max_length
        let mut result = String::new();
        let mut temp_path = PathBuf::new();

        for component in components.iter().rev() {
            temp_path = Path::new(component).join(&temp_path);
            let temp_str = format!(".../{}", temp_path.display());

            if temp_str.len() > max_length {
                break;
            }

            result = temp_str;
        }

        if result.is_empty() {
            // If we can't fit even with shortening, just truncate
            format!("...{}", &path_str[path_str.len().saturating_sub(max_length - 3)..])
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_directory_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let size = FileUtils::calculate_directory_size(temp_dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_directory_size_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let size = FileUtils::calculate_directory_size(temp_dir.path()).unwrap();
        assert!(size > 0);
        assert_eq!(size, 13); // "Hello, World!" is 13 bytes
    }

    #[test]
    fn test_calculate_directory_size_nonexistent() {
        let result = FileUtils::calculate_directory_size(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_can_delete_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let result = FileUtils::can_delete_directory(temp_dir.path());
        assert!(result.is_ok());
        // Note: The actual result depends on system permissions
    }

    #[test]
    fn test_can_delete_directory_nonexistent() {
        let result = FileUtils::can_delete_directory(Path::new("/nonexistent/path")).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_count_items_empty() {
        let temp_dir = TempDir::new().unwrap();
        let (files, dirs) = FileUtils::count_items(temp_dir.path()).unwrap();
        assert_eq!(files, 0);
        assert_eq!(dirs, 0);
    }

    #[test]
    fn test_count_items_with_content() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file
        fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

        // Create a subdirectory
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let (files, dirs) = FileUtils::count_items(temp_dir.path()).unwrap();
        assert_eq!(files, 1);
        assert_eq!(dirs, 1);
    }

    #[test]
    fn test_is_valid_venv_directory_false() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!FileUtils::is_valid_venv_directory(temp_dir.path()));
    }

    #[test]
    fn test_is_valid_venv_directory_wrong_name() {
        let temp_dir = TempDir::new().unwrap();
        let not_venv = temp_dir.path().join("not_venv");
        fs::create_dir(&not_venv).unwrap();
        assert!(!FileUtils::is_valid_venv_directory(&not_venv));
    }

    #[test]
    fn test_is_valid_venv_directory_true() {
        let temp_dir = TempDir::new().unwrap();
        let venv_dir = temp_dir.path().join(".venv");
        fs::create_dir(&venv_dir).unwrap();

        // Create some common venv items
        fs::create_dir(venv_dir.join("bin")).unwrap();
        fs::create_dir(venv_dir.join("lib")).unwrap();
        fs::write(venv_dir.join("pyvenv.cfg"), "content").unwrap();

        assert!(FileUtils::is_valid_venv_directory(&venv_dir));
    }

    #[test]
    fn test_format_path_for_display_short() {
        let path = Path::new("/short/path");
        let formatted = FileUtils::format_path_for_display(path, 100);
        assert_eq!(formatted, "/short/path");
    }

    #[test]
    fn test_format_path_for_display_long() {
        let path = Path::new("/very/long/path/that/exceeds/the/maximum/length/allowed");
        let formatted = FileUtils::format_path_for_display(path, 20);
        assert!(formatted.len() <= 20);
        assert!(formatted.starts_with("..."));
    }
}
