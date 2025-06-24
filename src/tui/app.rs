//! TUI Application State Management
//!
//! This module handles the application state for the TUI mode, including
//! .venv directory management, selection state, sorting, and user interactions.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use crate::core::{VenvInfo, Result};
use super::{AppState, SortBy};

/// Main application state for the TUI mode
pub struct TuiApp {
    /// Current application state
    state: AppState,
    /// List of found .venv directories
    venvs: Vec<VenvInfo>,
    /// Currently selected index in the list
    selected_index: usize,
    /// Set of selected .venv directories for deletion
    selected_venvs: HashSet<usize>,
    /// Current sorting method
    sort_by: SortBy,
    /// Reverse sort order
    reverse_sort: bool,
    /// Current status message
    status: String,
    /// Error message (if any)
    error_message: String,
    /// Last tick time for animations
    last_tick: Instant,
    /// Loading animation state
    loading_dots: usize,
    /// Deletion progress information
    deletion_progress: DeletionProgress,
    /// Current directory being browsed
    current_directory: PathBuf,
    /// Whether search is recursive
    is_recursive: bool,
    /// Whether to show hidden information
    show_details: bool,
    /// Scroll offset for the list
    scroll_offset: usize,
    /// Number of items visible in the list
    visible_items: usize,
}

/// Progress information for ongoing deletion operations
#[derive(Debug, Clone)]
pub struct DeletionProgress {
    /// Total number of items to delete
    pub total: usize,
    /// Number of items completed
    pub completed: usize,
    /// Number of successful deletions
    pub successful: usize,
    /// Number of failed deletions
    pub failed: usize,
    /// Results of deletion operations (path, success)
    pub results: Vec<(String, bool)>,
}

impl Default for DeletionProgress {
    fn default() -> Self {
        Self {
            total: 0,
            completed: 0,
            successful: 0,
            failed: 0,
            results: Vec::new(),
        }
    }
}

impl TuiApp {
    /// Create a new TUI application instance
    pub fn new() -> Self {
        Self {
            state: AppState::Loading,
            venvs: Vec::new(),
            selected_index: 0,
            selected_venvs: HashSet::new(),
            sort_by: SortBy::Path,
            reverse_sort: false,
            status: "Initializing...".to_string(),
            error_message: String::new(),
            last_tick: Instant::now(),
            loading_dots: 0,
            deletion_progress: DeletionProgress::default(),
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            is_recursive: true,
            show_details: false,
            scroll_offset: 0,
            visible_items: 20, // Default, will be updated based on terminal size
        }
    }

    /// Get the current application state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Set the application state
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    /// Get the list of .venv directories
    pub fn venvs(&self) -> &[VenvInfo] {
        &self.venvs
    }

    /// Set the list of .venv directories
    pub fn set_venvs(&mut self, mut venvs: Vec<VenvInfo>) {
        self.sort_venvs(&mut venvs);
        self.venvs = venvs;
        self.selected_index = 0;
        self.selected_venvs.clear();
        self.scroll_offset = 0;

        // Update status with current count
        if self.venvs.is_empty() {
            self.set_status("No .venv directories found".to_string());
        } else {
            self.set_status(format!("Found {} .venv directories", self.venvs.len()));
        }
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the currently selected .venv info
    pub fn selected_venv(&self) -> Option<&VenvInfo> {
        self.venvs.get(self.selected_index)
    }

    /// Get the set of selected .venv indices
    pub fn selected_venvs(&self) -> &HashSet<usize> {
        &self.selected_venvs
    }

    /// Get the selected .venv directories
    pub fn get_selected_venvs(&self) -> Vec<VenvInfo> {
        self.selected_venvs
            .iter()
            .filter_map(|&i| self.venvs.get(i))
            .cloned()
            .collect()
    }

    /// Check if there are any selected items
    pub fn has_selected_items(&self) -> bool {
        !self.selected_venvs.is_empty()
    }

    /// Get the current sorting method
    pub fn sort_by(&self) -> SortBy {
        self.sort_by
    }

    /// Get the current status message
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Set the status message
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    /// Get the error message
    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    /// Set the error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = error;
    }

    /// Get the current directory
    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    /// Set the current directory and recursive status
    pub fn set_current_directory(&mut self, path: PathBuf, recursive: bool) {
        self.current_directory = path;
        self.is_recursive = recursive;
    }

    /// Check if search is recursive
    pub fn is_recursive(&self) -> bool {
        self.is_recursive
    }

    /// Get the loading animation state
    pub fn loading_dots(&self) -> usize {
        self.loading_dots
    }

    /// Get deletion progress information
    pub fn deletion_progress(&self) -> &DeletionProgress {
        &self.deletion_progress
    }

    /// Check if details should be shown
    pub fn show_details(&self) -> bool {
        self.show_details
    }

    /// Toggle details visibility
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set the number of visible items (based on terminal size)
    pub fn set_visible_items(&mut self, count: usize) {
        self.visible_items = count;
    }

    /// Get the number of visible items
    pub fn visible_items(&self) -> usize {
        self.visible_items
    }

    /// Move selection to the next item
    pub fn select_next(&mut self) {
        if !self.venvs.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.venvs.len() - 1);
            self.adjust_scroll();
        }
    }

    /// Move selection to the previous item
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
        }
    }

    /// Move selection to the first item
    pub fn select_first(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Move selection to the last item
    pub fn select_last(&mut self) {
        if !self.venvs.is_empty() {
            self.selected_index = self.venvs.len() - 1;
            self.adjust_scroll();
        }
    }

    /// Move selection up by a page
    pub fn page_up(&mut self) {
        let page_size = self.visible_items.saturating_sub(1);
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.adjust_scroll();
    }

    /// Move selection down by a page
    pub fn page_down(&mut self) {
        if !self.venvs.is_empty() {
            let page_size = self.visible_items.saturating_sub(1);
            self.selected_index = (self.selected_index + page_size).min(self.venvs.len() - 1);
            self.adjust_scroll();
        }
    }

    /// Toggle selection of the current item
    pub fn toggle_selected(&mut self) {
        if self.selected_venvs.contains(&self.selected_index) {
            self.selected_venvs.remove(&self.selected_index);
        } else {
            self.selected_venvs.insert(self.selected_index);
        }
    }

    /// Select all items
    pub fn select_all(&mut self) {
        self.selected_venvs = (0..self.venvs.len()).collect();
    }

    /// Deselect all items
    pub fn deselect_all(&mut self) {
        self.selected_venvs.clear();
    }

    /// Cycle through sorting options
    pub fn cycle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
        self.sort_current_venvs();
    }

    /// Reverse the current sort order
    pub fn reverse_sort(&mut self) {
        self.reverse_sort = !self.reverse_sort;
        self.sort_current_venvs();
    }

    /// Sort the current list of venvs
    fn sort_current_venvs(&mut self) {
        let mut venvs_copy = self.venvs.clone();
        self.sort_venvs(&mut venvs_copy);
        self.venvs = venvs_copy;
        // Reset selection to maintain valid state
        self.selected_index = self.selected_index.min(self.venvs.len().saturating_sub(1));
        self.adjust_scroll();
    }

    /// Sort a list of venvs according to current settings
    fn sort_venvs(&self, venvs: &mut [VenvInfo]) {
        match self.sort_by {
            SortBy::Path => {
                venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        b.path().cmp(a.path())
                    } else {
                        a.path().cmp(b.path())
                    }
                });
            }
            SortBy::Size => {
                venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.size_bytes().cmp(&b.size_bytes())
                    } else {
                        b.size_bytes().cmp(&a.size_bytes())
                    }
                });
            }
            SortBy::Created => {
                venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.created().cmp(b.created())
                    } else {
                        b.created().cmp(a.created())
                    }
                });
            }
            SortBy::LastModified => {
                venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.last_modified().cmp(b.last_modified())
                    } else {
                        b.last_modified().cmp(a.last_modified())
                    }
                });
            }
        }
    }

    /// Adjust scroll offset to keep selected item visible
    fn adjust_scroll(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_items {
            self.scroll_offset = self.selected_index.saturating_sub(self.visible_items - 1);
        }
    }

    /// Handle periodic tick for animations and updates
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick).as_millis() > 500 {
            self.last_tick = now;
            self.loading_dots = (self.loading_dots + 1) % 4;
        }
    }

    /// Handle deletion results
    pub fn handle_deletion_results(&mut self, results: Vec<(VenvInfo, Result<()>)>) {
        let mut successful = 0;
        let mut failed = 0;
        let mut simplified_results = Vec::new();

        for (venv, result) in results {
            match result {
                Ok(_) => {
                    successful += 1;
                    simplified_results.push((venv.path().display().to_string(), true));
                }
                Err(_) => {
                    failed += 1;
                    simplified_results.push((venv.path().display().to_string(), false));
                }
            }
        }

        self.deletion_progress = DeletionProgress {
            total: simplified_results.len(),
            completed: simplified_results.len(),
            successful,
            failed,
            results: simplified_results,
        };

        // Update status message with more detail
        if failed == 0 {
            self.set_status(format!("✅ Successfully deleted {} directories. List will refresh automatically.", successful));
        } else {
            self.set_status(format!("⚠️ Deleted {} directories, {} failed. Check permissions for failed items.", successful, failed));
        }

        // Clear selected items after deletion
        self.selected_venvs.clear();

        // Reset selection to first item
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Open the folder containing the selected .venv
    pub fn open_folder(&self) -> Result<()> {
        if let Some(venv) = self.selected_venv() {
            if let Some(parent) = venv.parent_path() {
                // Try to open the folder using the system's default file manager
                #[cfg(target_os = "macos")]
                std::process::Command::new("open")
                    .arg(parent)
                    .spawn()?;

                #[cfg(target_os = "linux")]
                std::process::Command::new("xdg-open")
                    .arg(parent)
                    .spawn()?;

                #[cfg(target_os = "windows")]
                std::process::Command::new("explorer")
                    .arg(parent)
                    .spawn()?;
            }
        }
        Ok(())
    }

    /// Get the visible range of items for the current scroll position
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.visible_items).min(self.venvs.len());
        (start, end)
    }

    /// Check if an item is currently selected for deletion
    pub fn is_item_selected(&self, index: usize) -> bool {
        self.selected_venvs.contains(&index)
    }

    /// Get summary statistics for the current .venv list
    pub fn get_summary_stats(&self) -> SummaryStats {
        let total_size = self.venvs.iter().map(|v| v.size_bytes()).sum();
        let selected_size = self.selected_venvs
            .iter()
            .filter_map(|&i| self.venvs.get(i))
            .map(|v| v.size_bytes())
            .sum();

        let old_count = self.venvs.iter().filter(|v| v.is_old()).count();
        let recent_count = self.venvs.iter().filter(|v| v.is_recently_used()).count();

        SummaryStats {
            total_count: self.venvs.len(),
            selected_count: self.selected_venvs.len(),
            total_size,
            selected_size,
            old_count,
            recent_count,
        }
    }
}

/// Summary statistics for the .venv list
#[derive(Debug, Clone)]
pub struct SummaryStats {
    pub total_count: usize,
    pub selected_count: usize,
    pub total_size: u64,
    pub selected_size: u64,
    pub old_count: usize,
    pub recent_count: usize,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Local;

    fn create_test_venv(path: &str, size: u64) -> VenvInfo {
        let now = Local::now();
        VenvInfo::new(
            PathBuf::from(path),
            size,
            now,
            now,
        )
    }

    #[test]
    fn test_app_initialization() {
        let app = TuiApp::new();
        assert_eq!(app.state(), &AppState::Loading);
        assert_eq!(app.venvs().len(), 0);
        assert_eq!(app.selected_index(), 0);
        assert!(!app.has_selected_items());
    }

    #[test]
    fn test_selection_navigation() {
        let mut app = TuiApp::new();
        let venvs = vec![
            create_test_venv("/path1/.venv", 100),
            create_test_venv("/path2/.venv", 200),
            create_test_venv("/path3/.venv", 300),
        ];
        app.set_venvs(venvs);

        assert_eq!(app.selected_index(), 0);

        app.select_next();
        assert_eq!(app.selected_index(), 1);

        app.select_next();
        assert_eq!(app.selected_index(), 2);

        app.select_next(); // Should stay at last item
        assert_eq!(app.selected_index(), 2);

        app.select_previous();
        assert_eq!(app.selected_index(), 1);

        app.select_first();
        assert_eq!(app.selected_index(), 0);

        app.select_last();
        assert_eq!(app.selected_index(), 2);
    }

    #[test]
    fn test_item_selection() {
        let mut app = TuiApp::new();
        let venvs = vec![
            create_test_venv("/path1/.venv", 100),
            create_test_venv("/path2/.venv", 200),
        ];
        app.set_venvs(venvs);

        assert!(!app.has_selected_items());

        app.toggle_selected();
        assert!(app.has_selected_items());
        assert!(app.is_item_selected(0));

        app.select_next();
        app.toggle_selected();
        assert_eq!(app.selected_venvs().len(), 2);

        app.deselect_all();
        assert!(!app.has_selected_items());

        app.select_all();
        assert_eq!(app.selected_venvs().len(), 2);
    }

    #[test]
    fn test_sorting() {
        let mut app = TuiApp::new();
        assert_eq!(app.sort_by(), SortBy::Path);

        app.cycle_sort();
        assert_eq!(app.sort_by(), SortBy::Size);

        app.cycle_sort();
        assert_eq!(app.sort_by(), SortBy::Created);

        app.cycle_sort();
        assert_eq!(app.sort_by(), SortBy::LastModified);

        app.cycle_sort();
        assert_eq!(app.sort_by(), SortBy::Path);
    }
}
