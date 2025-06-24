//! VenvInfo module for storing and managing information about .venv directories
//!
//! This module contains the VenvInfo struct which holds all relevant information
//! about a Python virtual environment directory, including path, size, and timestamps.

use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};
use std::fmt;

/// Information about a Python virtual environment directory
#[derive(Debug, Clone, PartialEq)]
pub struct VenvInfo {
    /// Full path to the .venv directory
    path: PathBuf,
    /// Size of the directory in bytes
    size_bytes: u64,
    /// When the directory was created
    created: DateTime<Local>,
    /// When the directory was last modified (last used)
    last_modified: DateTime<Local>,
}

impl VenvInfo {
    /// Create a new VenvInfo instance
    ///
    /// # Arguments
    /// * `path` - Full path to the .venv directory
    /// * `size_bytes` - Size of the directory in bytes
    /// * `created` - Creation timestamp
    /// * `last_modified` - Last modification timestamp
    ///
    /// # Returns
    /// A new VenvInfo instance
    pub fn new(
        path: PathBuf,
        size_bytes: u64,
        created: DateTime<Local>,
        last_modified: DateTime<Local>,
    ) -> Self {
        Self {
            path,
            size_bytes,
            created,
            last_modified,
        }
    }

    /// Get the path to the .venv directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the parent directory of the .venv folder
    pub fn parent_path(&self) -> Option<&Path> {
        self.path.parent()
    }

    /// Get the size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Get the size formatted as a human-readable string
    ///
    /// # Returns
    /// Size formatted as MB or GB depending on the size
    pub fn size_formatted(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.2} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.2} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.2} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size_bytes)
        }
    }

    /// Get the creation timestamp
    pub fn created(&self) -> &DateTime<Local> {
        &self.created
    }

    /// Get the creation date formatted as a string
    pub fn created_formatted(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get the last modified timestamp
    pub fn last_modified(&self) -> &DateTime<Local> {
        &self.last_modified
    }

    /// Get the last modified date formatted as a string
    pub fn last_modified_formatted(&self) -> String {
        self.last_modified.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get the project name (directory name containing the .venv)
    pub fn project_name(&self) -> Option<String> {
        self.parent_path()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    /// Check if this .venv was recently used (within the last 30 days)
    pub fn is_recently_used(&self) -> bool {
        let now = Local::now();
        let thirty_days_ago = now - chrono::Duration::days(30);
        self.last_modified > thirty_days_ago
    }

    /// Check if this .venv is old (not modified in the last 90 days)
    pub fn is_old(&self) -> bool {
        let now = Local::now();
        let ninety_days_ago = now - chrono::Duration::days(90);
        self.last_modified < ninety_days_ago
    }

    /// Get age in days since last modification
    pub fn age_in_days(&self) -> i64 {
        let now = Local::now();
        (now - self.last_modified).num_days()
    }

    /// Get a summary string for display purposes
    pub fn summary(&self) -> String {
        format!(
            "{} | {} | Created: {} | Last used: {}",
            self.path.display(),
            self.size_formatted(),
            self.created_formatted(),
            self.last_modified_formatted()
        )
    }

    /// Get location string (parent directory path)
    pub fn location(&self) -> String {
        self.parent_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

impl fmt::Display for VenvInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VenvInfo {{ path: {}, size: {}, created: {}, last_modified: {} }}",
            self.path.display(),
            self.size_formatted(),
            self.created_formatted(),
            self.last_modified_formatted()
        )
    }
}

/// Comparison implementations for sorting VenvInfo instances
impl VenvInfo {
    /// Compare by size (largest first)
    pub fn compare_by_size(&self, other: &Self) -> std::cmp::Ordering {
        other.size_bytes.cmp(&self.size_bytes)
    }

    /// Compare by creation date (newest first)
    pub fn compare_by_created(&self, other: &Self) -> std::cmp::Ordering {
        other.created.cmp(&self.created)
    }

    /// Compare by last modified date (most recently used first)
    pub fn compare_by_last_modified(&self, other: &Self) -> std::cmp::Ordering {
        other.last_modified.cmp(&self.last_modified)
    }

    /// Compare by path (alphabetical)
    pub fn compare_by_path(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_venv_info() -> VenvInfo {
        let path = PathBuf::from("/home/user/project/.venv");
        let size = 1024 * 1024 * 100; // 100 MB
        let created = Local::now() - chrono::Duration::days(10);
        let modified = Local::now() - chrono::Duration::days(5);

        VenvInfo::new(path, size, created, modified)
    }

    #[test]
    fn test_venv_info_creation() {
        let venv_info = create_test_venv_info();
        assert_eq!(venv_info.path(), Path::new("/home/user/project/.venv"));
        assert_eq!(venv_info.size_bytes(), 1024 * 1024 * 100);
    }

    #[test]
    fn test_size_formatting() {
        let venv_info = create_test_venv_info();
        let formatted = venv_info.size_formatted();
        assert!(formatted.contains("MB"));
    }

    #[test]
    fn test_size_formatting_gb() {
        let path = PathBuf::from("/test/.venv");
        let size = 1024 * 1024 * 1024 * 2; // 2 GB
        let now = Local::now();
        let venv_info = VenvInfo::new(path, size, now, now);

        let formatted = venv_info.size_formatted();
        assert!(formatted.contains("GB"));
    }

    #[test]
    fn test_size_formatting_kb() {
        let path = PathBuf::from("/test/.venv");
        let size = 1024 * 500; // 500 KB
        let now = Local::now();
        let venv_info = VenvInfo::new(path, size, now, now);

        let formatted = venv_info.size_formatted();
        assert!(formatted.contains("KB"));
    }

    #[test]
    fn test_size_formatting_bytes() {
        let path = PathBuf::from("/test/.venv");
        let size = 512; // 512 bytes
        let now = Local::now();
        let venv_info = VenvInfo::new(path, size, now, now);

        let formatted = venv_info.size_formatted();
        assert!(formatted.contains("bytes"));
    }

    #[test]
    fn test_project_name() {
        let venv_info = create_test_venv_info();
        assert_eq!(venv_info.project_name(), Some("project".to_string()));
    }

    #[test]
    fn test_is_recently_used() {
        let path = PathBuf::from("/test/.venv");
        let now = Local::now();
        let recent = now - chrono::Duration::days(10);
        let venv_info = VenvInfo::new(path, 1024, now, recent);

        assert!(venv_info.is_recently_used());
    }

    #[test]
    fn test_is_old() {
        let path = PathBuf::from("/test/.venv");
        let now = Local::now();
        let old = now - chrono::Duration::days(100);
        let venv_info = VenvInfo::new(path, 1024, now, old);

        assert!(venv_info.is_old());
    }

    #[test]
    fn test_age_calculation() {
        let path = PathBuf::from("/test/.venv");
        let now = Local::now();
        let modified = now - chrono::Duration::days(15);
        let venv_info = VenvInfo::new(path, 1024, now, modified);

        assert_eq!(venv_info.age_in_days(), 15);
    }

    #[test]
    fn test_display_formatting() {
        let venv_info = create_test_venv_info();
        let display_str = format!("{}", venv_info);
        assert!(display_str.contains("VenvInfo"));
        assert!(display_str.contains("/home/user/project/.venv"));
    }

    #[test]
    fn test_location() {
        let venv_info = create_test_venv_info();
        assert_eq!(venv_info.location(), "/home/user/project");
    }
}
