//! TUI module for VenvCleaner
//!
//! This module handles the Terminal User Interface for interactive .venv directory management.
//! It provides a full-screen terminal interface using ratatui with keyboard navigation,
//! sorting options, and interactive deletion capabilities.

use std::io;
use std::time::Duration;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use log::info;

use crate::core::{VenvCleaner, Result};

pub mod app;
pub mod ui;
pub mod events;

pub use app::TuiApp;
pub use events::{AppEvent, EventHandler};

/// Main TUI mode handler for VenvCleaner
pub struct TuiMode {
    /// The core VenvCleaner instance
    cleaner: VenvCleaner,
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Application state
    app: TuiApp,
    /// Event handler for user input
    event_handler: EventHandler,
}

/// Application states for the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
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
    /// Showing help screen
    Help,
    /// Application should quit
    Quit,
}

/// Sorting options for .venv directories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    /// Sort by path (alphabetical)
    Path,
    /// Sort by size (largest first)
    Size,
    /// Sort by creation date (newest first)
    Created,
    /// Sort by last modified date (most recent first)
    LastModified,
}

impl SortBy {
    /// Get the next sort option in sequence
    pub fn next(self) -> Self {
        match self {
            SortBy::Path => SortBy::Size,
            SortBy::Size => SortBy::Created,
            SortBy::Created => SortBy::LastModified,
            SortBy::LastModified => SortBy::Path,
        }
    }

    /// Get the previous sort option in sequence
    pub fn previous(self) -> Self {
        match self {
            SortBy::Path => SortBy::LastModified,
            SortBy::Size => SortBy::Path,
            SortBy::Created => SortBy::Size,
            SortBy::LastModified => SortBy::Created,
        }
    }

    /// Get the display name for this sort option
    pub fn display_name(self) -> &'static str {
        match self {
            SortBy::Path => "Path",
            SortBy::Size => "Size",
            SortBy::Created => "Created",
            SortBy::LastModified => "Last Used",
        }
    }
}

impl TuiMode {
    /// Create a new TuiMode instance
    pub fn new(
        base_directory: std::path::PathBuf,
        recursive: bool,
        verbosity: u8,
    ) -> Result<Self> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create VenvCleaner instance
        let cleaner = VenvCleaner::new(
            base_directory.clone(),
            recursive,
            false, // force_mode = false for TUI
            false, // dry_run = false (we handle this in TUI)
            verbosity,
        );

        // Create application state
        let mut app = TuiApp::new();
        app.set_current_directory(base_directory, recursive);

        // Create event handler
        let event_handler = EventHandler::new(Duration::from_millis(250))?;

        Ok(Self {
            cleaner,
            terminal,
            app,
            event_handler,
        })
    }

    /// Run the TUI application
    pub fn run(&mut self) -> Result<()> {
        info!("Starting TUI mode");

        // Set initial loading state before drawing anything
        self.app.set_state(AppState::Loading);
        self.app.set_status("Initializing VenvCleaner TUI...".to_string());

        // Draw initial loading screen immediately
        self.terminal.draw(|f| {
            let size = f.size();
            ui::draw_loading_screen(f, size, &self.app);
        })?;

        // Start loading .venv directories in the background
        self.start_loading_venvs()?;

        // Main event loop
        loop {
            // Draw the UI
            let app_ref = &self.app;
            self.terminal.draw(|f| {
                let size = f.size();
                match app_ref.state() {
                    AppState::Loading => {
                        ui::draw_loading_screen(f, size, app_ref);
                    }
                    AppState::Browsing => {
                        ui::draw_main_screen(f, size, app_ref);
                    }
                    AppState::ConfirmingDeletion => {
                        ui::draw_main_screen(f, size, app_ref);
                        ui::draw_confirmation_dialog(f, size, app_ref);
                    }
                    AppState::Deleting => {
                        ui::draw_main_screen(f, size, app_ref);
                        ui::draw_deletion_progress(f, size, app_ref);
                    }
                    AppState::Error => {
                        ui::draw_error_screen(f, size, app_ref);
                    }
                    AppState::Help => {
                        ui::draw_help_screen(f, size);
                    }
                    AppState::Quit => {
                        // Should not reach here
                    }
                }
            })?;

            // Handle events
            if let Ok(event) = self.event_handler.next() {
                match event {
                    AppEvent::Input(key_event) => {
                        if self.handle_key_event(key_event)? {
                            break; // Exit requested
                        }
                    }
                    AppEvent::Tick => {
                        self.handle_tick()?;
                    }
                    AppEvent::VenvsLoaded(venvs) => {
                        self.app.set_venvs(venvs);
                        self.app.set_state(AppState::Browsing);
                    }
                    AppEvent::LoadError(error) => {
                        self.app.set_error(error);
                        self.app.set_state(AppState::Error);
                    }
                    AppEvent::DeletionComplete(results) => {
                        self.app.handle_deletion_results(results);
                        // Refresh the list by reloading after a short delay to show completion
                        self.app.set_state(AppState::Loading);
                        self.start_loading_venvs()?;
                    }
                }
            }
        }

        self.cleanup()?;
        Ok(())
    }

    /// Start loading .venv directories in the background
    fn start_loading_venvs(&mut self) -> Result<()> {
        self.app.set_state(AppState::Loading);
        let search_mode = if self.cleaner.is_recursive() { "recursively" } else { "in current directory" };
        self.app.set_status(format!("🔍 Scanning for .venv directories {}...", search_mode));

        // Draw initial scanning message
        self.terminal.draw(|f| {
            let size = f.size();
            ui::draw_loading_screen(f, size, &self.app);
        })?;

        // Update status to show we're analyzing directories
        self.app.set_status(format!("📁 Analyzing directories {}...", search_mode));

        // Draw updated status
        self.terminal.draw(|f| {
            let size = f.size();
            ui::draw_loading_screen(f, size, &self.app);
        })?;

        // In a real implementation, this would spawn a background task
        // For now, we'll do it synchronously but show the loading state

        // Add a brief delay to show the scanning message
        std::thread::sleep(std::time::Duration::from_millis(200));

        let venvs = self.cleaner.find_venv_directories();

        match venvs {
            Ok(venvs) => {
                // Show completion message briefly
                self.app.set_status(format!("✅ Scan complete! Processing {} directories...", venvs.len()));
                self.terminal.draw(|f| {
                    let size = f.size();
                    ui::draw_loading_screen(f, size, &self.app);
                })?;

                // Brief delay to show completion message
                std::thread::sleep(std::time::Duration::from_millis(300));

                self.app.set_venvs(venvs);
                self.app.set_state(AppState::Browsing);
                let count = self.app.venvs().len();
                if count == 0 {
                    self.app.set_status("No .venv directories found. Press 'r' to refresh or 'q' to quit.".to_string());
                } else {
                    self.app.set_status(format!("Found {} .venv directories. Use arrow keys to navigate, Space to select.", count));
                }
            }
            Err(e) => {
                self.app.set_error(e.to_string());
                self.app.set_state(AppState::Error);
            }
        }

        Ok(())
    }

    /// Handle keyboard input events
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match self.app.state() {
            AppState::Loading => {
                // Only allow quit during loading
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    return Ok(true);
                }
            }
            AppState::Browsing => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                    KeyCode::Char('h') | KeyCode::F(1) => {
                        self.app.set_state(AppState::Help);
                    }
                    KeyCode::Char('r') => {
                        self.start_loading_venvs()?;
                    }
                    KeyCode::Up => {
                        self.app.select_previous();
                    }
                    KeyCode::Down => {
                        self.app.select_next();
                    }
                    KeyCode::Home => {
                        self.app.select_first();
                    }
                    KeyCode::End => {
                        self.app.select_last();
                    }
                    KeyCode::PageUp => {
                        self.app.page_up();
                    }
                    KeyCode::PageDown => {
                        self.app.page_down();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.app.toggle_selected();
                    }
                    KeyCode::Char('a') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.app.select_all();
                        }
                    }
                    KeyCode::Char('d') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.app.deselect_all();
                        }
                    }
                    KeyCode::Delete | KeyCode::Char('x') => {
                        if self.app.has_selected_items() {
                            self.app.set_state(AppState::ConfirmingDeletion);
                        }
                    }
                    KeyCode::Char('s') => {
                        self.app.cycle_sort();
                        self.app.set_status(format!("Sorted by {}", self.app.sort_by().display_name()));
                    }
                    KeyCode::Char('o') => {
                        self.app.open_folder()?;
                    }
                    _ => {}
                }
            }
            AppState::ConfirmingDeletion => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        self.start_deletion()?;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.app.set_state(AppState::Browsing);
                    }
                    _ => {}
                }
            }
            AppState::Deleting => {
                // Only allow force quit during deletion
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(true);
                }
            }
            AppState::Error => {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        self.app.set_state(AppState::Browsing);
                    }
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppState::Help => {
                // Any key exits help
                self.app.set_state(AppState::Browsing);
            }
            AppState::Quit => {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Handle tick events (periodic updates)
    fn handle_tick(&mut self) -> Result<()> {
        // Update any animations or periodic state changes
        self.app.tick();
        Ok(())
    }

    /// Start the deletion process for selected .venv directories
    fn start_deletion(&mut self) -> Result<()> {
        self.app.set_state(AppState::Deleting);
        self.app.set_status("Deleting selected directories...".to_string());

        let selected_venvs = self.app.get_selected_venvs();
        let mut results = Vec::new();

        for venv in &selected_venvs {
            let result = self.cleaner.delete_venv_directory(venv);
            results.push((venv.clone(), result));
        }

        // Calculate stats before moving results
        let deleted_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed_count = results.len() - deleted_count;

        self.app.handle_deletion_results(results);

        // Set a brief completion message

        if failed_count == 0 {
            self.app.set_status(format!("Successfully deleted {} directories. Refreshing list...", deleted_count));
        } else {
            self.app.set_status(format!("Deleted {} directories, {} failed. Refreshing list...", deleted_count, failed_count));
        }

        // Trigger a refresh by going back to loading state
        self.app.set_state(AppState::Loading);
        self.start_loading_venvs()?;

        Ok(())
    }

    /// Draw the user interface
    fn draw_ui(&self, f: &mut ratatui::Frame) {
        let size = f.size();

        match self.app.state() {
            AppState::Loading => {
                ui::draw_loading_screen(f, size, &self.app);
            }
            AppState::Browsing => {
                ui::draw_main_screen(f, size, &self.app);
            }
            AppState::ConfirmingDeletion => {
                ui::draw_main_screen(f, size, &self.app);
                ui::draw_confirmation_dialog(f, size, &self.app);
            }
            AppState::Deleting => {
                ui::draw_main_screen(f, size, &self.app);
                ui::draw_deletion_progress(f, size, &self.app);
            }
            AppState::Error => {
                ui::draw_error_screen(f, size, &self.app);
            }
            AppState::Help => {
                ui::draw_help_screen(f, size);
            }
            AppState::Quit => {
                // Should not reach here
            }
        }
    }

    /// Clean up terminal state before exiting
    fn cleanup(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TuiMode {
    fn drop(&mut self) {
        // Ensure cleanup happens even if there's a panic
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sort_by_cycle() {
        assert_eq!(SortBy::Path.next(), SortBy::Size);
        assert_eq!(SortBy::Size.next(), SortBy::Created);
        assert_eq!(SortBy::Created.next(), SortBy::LastModified);
        assert_eq!(SortBy::LastModified.next(), SortBy::Path);
    }

    #[test]
    fn test_sort_by_previous() {
        assert_eq!(SortBy::Path.previous(), SortBy::LastModified);
        assert_eq!(SortBy::Size.previous(), SortBy::Path);
        assert_eq!(SortBy::Created.previous(), SortBy::Size);
        assert_eq!(SortBy::LastModified.previous(), SortBy::Created);
    }

    #[test]
    fn test_sort_by_display_names() {
        assert_eq!(SortBy::Path.display_name(), "Path");
        assert_eq!(SortBy::Size.display_name(), "Size");
        assert_eq!(SortBy::Created.display_name(), "Created");
        assert_eq!(SortBy::LastModified.display_name(), "Last Used");
    }
}
