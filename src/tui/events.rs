//! Event handling for the TUI mode
//!
//! This module handles keyboard input, background events, and async operations
//! for the TUI interface. It provides a unified event system that can handle
//! user input and background tasks like loading .venv directories.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyEvent};
use log::{debug, warn};

use crate::core::{VenvInfo, VenvCleanerError, Result};

/// Events that can occur in the TUI application
#[derive(Debug)]
pub enum AppEvent {
    /// Keyboard input event
    Input(KeyEvent),
    /// Periodic tick for animations and updates
    Tick,
    /// .venv directories have been loaded
    VenvsLoaded(Vec<VenvInfo>),
    /// Error occurred while loading .venv directories
    LoadError(String),
    /// Deletion operation completed
    DeletionComplete(Vec<(VenvInfo, Result<()>)>),
}

/// Event handler for the TUI application
pub struct EventHandler {
    /// Receiver for application events
    receiver: Receiver<AppEvent>,
    /// Sender for application events (used by background tasks)
    sender: Sender<AppEvent>,
    /// Last tick time
    last_tick: Instant,
    /// Tick interval
    tick_interval: Duration,
}

impl EventHandler {
    /// Create a new event handler
    ///
    /// # Arguments
    /// * `tick_interval` - Interval between tick events
    ///
    /// # Returns
    /// A new EventHandler instance or an error
    pub fn new(tick_interval: Duration) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();

        // Start the input event thread
        let input_sender = sender.clone();
        thread::spawn(move || {
            loop {
                match event::poll(Duration::from_millis(100)) {
                    Ok(true) => {
                        if let Ok(event) = event::read() {
                            match event {
                                Event::Key(key) => {
                                    if input_sender.send(AppEvent::Input(key)).is_err() {
                                        break; // Receiver dropped, exit thread
                                    }
                                }
                                Event::Resize(_, _) => {
                                    // Handle resize events if needed
                                    debug!("Terminal resized");
                                }
                                Event::Mouse(_) => {
                                    // Mouse events are currently not handled
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(false) => {
                        // No events available, continue polling
                    }
                    Err(e) => {
                        warn!("Error polling for events: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            receiver,
            sender,
            last_tick: Instant::now(),
            tick_interval,
        })
    }

    /// Get the next event, blocking until one is available
    ///
    /// # Returns
    /// The next AppEvent or an error
    pub fn next(&mut self) -> Result<AppEvent> {
        // Check if it's time for a tick event
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_interval {
            self.last_tick = now;
            // Send tick event first if it's time
            if let Ok(event) = self.receiver.try_recv() {
                // If there's a pending event, return it first
                Ok(event)
            } else {
                // No pending events, return tick
                Ok(AppEvent::Tick)
            }
        } else {
            // Wait for the next event
            self.receiver.recv()
                .map_err(|_| VenvCleanerError::Io(
                    "Event channel closed".to_string()
                ))
        }
    }

    /// Try to get the next event without blocking
    ///
    /// # Returns
    /// Some(AppEvent) if an event is available, None otherwise
    pub fn try_next(&mut self) -> Option<AppEvent> {
        // Check for tick event first
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_interval {
            self.last_tick = now;
            return Some(AppEvent::Tick);
        }

        // Try to receive a non-blocking event
        self.receiver.try_recv().ok()
    }

    /// Get a sender for background tasks to send events
    pub fn sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }

    /// Start a background task to load .venv directories
    ///
    /// # Arguments
    /// * `base_path` - Base directory to search from
    /// * `recursive` - Whether to search recursively
    /// * `cleaner` - VenvCleaner instance to use for searching
    pub fn start_venv_loading_task(
        &self,
        _base_path: std::path::PathBuf,
        _recursive: bool,
        cleaner: std::sync::Arc<crate::core::VenvCleaner>,
    ) {
        let sender = self.sender.clone();

        thread::spawn(move || {
            debug!("Starting .venv loading task");

            match cleaner.find_venv_directories() {
                Ok(venvs) => {
                    debug!("Found {} .venv directories", venvs.len());
                    if sender.send(AppEvent::VenvsLoaded(venvs)).is_err() {
                        warn!("Failed to send VenvsLoaded event");
                    }
                }
                Err(e) => {
                    warn!("Error loading .venv directories: {}", e);
                    if sender.send(AppEvent::LoadError(e.to_string())).is_err() {
                        warn!("Failed to send LoadError event");
                    }
                }
            }
        });
    }

    /// Start a background task to delete selected .venv directories
    ///
    /// # Arguments
    /// * `venvs` - List of .venv directories to delete
    /// * `cleaner` - VenvCleaner instance to use for deletion
    pub fn start_deletion_task(
        &self,
        venvs: Vec<VenvInfo>,
        cleaner: std::sync::Arc<crate::core::VenvCleaner>,
    ) {
        let sender = self.sender.clone();

        thread::spawn(move || {
            debug!("Starting deletion task for {} directories", venvs.len());

            let mut results = Vec::new();

            for venv in venvs {
                let result = cleaner.delete_venv_directory(&venv);
                results.push((venv, result));
            }

            debug!("Deletion task completed");
            if sender.send(AppEvent::DeletionComplete(results)).is_err() {
                warn!("Failed to send DeletionComplete event");
            }
        });
    }
}

/// Helper trait for handling keyboard shortcuts
pub trait KeyboardShortcuts {
    /// Check if a key event matches a specific shortcut
    fn matches_shortcut(&self, key: &KeyEvent) -> bool;

    /// Get the display string for this shortcut
    fn display_string(&self) -> String;
}

/// Common keyboard shortcuts used in the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shortcut {
    /// Quit the application
    Quit,
    /// Show help
    Help,
    /// Refresh the list
    Refresh,
    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Move to first item
    Home,
    /// Move to last item
    End,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Toggle selection
    Toggle,
    /// Select all
    SelectAll,
    /// Deselect all
    DeselectAll,
    /// Delete selected items
    Delete,
    /// Sort by different criteria
    Sort,
    /// Open folder
    OpenFolder,
    /// Confirm action
    Confirm,
    /// Cancel action
    Cancel,
}

impl KeyboardShortcuts for Shortcut {
    fn matches_shortcut(&self, key: &KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        match self {
            Shortcut::Quit => {
                matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            }
            Shortcut::Help => {
                matches!(key.code, KeyCode::Char('h') | KeyCode::F(1))
            }
            Shortcut::Refresh => {
                matches!(key.code, KeyCode::Char('r'))
            }
            Shortcut::Up => {
                matches!(key.code, KeyCode::Up)
            }
            Shortcut::Down => {
                matches!(key.code, KeyCode::Down)
            }
            Shortcut::Home => {
                matches!(key.code, KeyCode::Home)
            }
            Shortcut::End => {
                matches!(key.code, KeyCode::End)
            }
            Shortcut::PageUp => {
                matches!(key.code, KeyCode::PageUp)
            }
            Shortcut::PageDown => {
                matches!(key.code, KeyCode::PageDown)
            }
            Shortcut::Toggle => {
                matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
            }
            Shortcut::SelectAll => {
                key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL)
            }
            Shortcut::DeselectAll => {
                key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL)
            }
            Shortcut::Delete => {
                matches!(key.code, KeyCode::Delete | KeyCode::Char('x'))
            }
            Shortcut::Sort => {
                matches!(key.code, KeyCode::Char('s'))
            }
            Shortcut::OpenFolder => {
                matches!(key.code, KeyCode::Char('o'))
            }
            Shortcut::Confirm => {
                matches!(key.code, KeyCode::Char('y') | KeyCode::Enter)
            }
            Shortcut::Cancel => {
                matches!(key.code, KeyCode::Char('n') | KeyCode::Esc)
            }
        }
    }

    fn display_string(&self) -> String {
        match self {
            Shortcut::Quit => "q/Esc".to_string(),
            Shortcut::Help => "h/F1".to_string(),
            Shortcut::Refresh => "r".to_string(),
            Shortcut::Up => "↑".to_string(),
            Shortcut::Down => "↓".to_string(),
            Shortcut::Home => "Home".to_string(),
            Shortcut::End => "End".to_string(),
            Shortcut::PageUp => "PgUp".to_string(),
            Shortcut::PageDown => "PgDn".to_string(),
            Shortcut::Toggle => "Space/Enter".to_string(),
            Shortcut::SelectAll => "Ctrl+A".to_string(),
            Shortcut::DeselectAll => "Ctrl+D".to_string(),
            Shortcut::Delete => "Del/x".to_string(),
            Shortcut::Sort => "s".to_string(),
            Shortcut::OpenFolder => "o".to_string(),
            Shortcut::Confirm => "y/Enter".to_string(),
            Shortcut::Cancel => "n/Esc".to_string(),
        }
    }
}

/// Get all available shortcuts for the current application state
pub fn get_shortcuts_for_state(state: &crate::tui::AppState) -> Vec<Shortcut> {
    use crate::tui::AppState;

    match state {
        AppState::Loading => vec![
            Shortcut::Quit,
        ],
        AppState::Browsing => vec![
            Shortcut::Quit,
            Shortcut::Help,
            Shortcut::Refresh,
            Shortcut::Up,
            Shortcut::Down,
            Shortcut::Home,
            Shortcut::End,
            Shortcut::PageUp,
            Shortcut::PageDown,
            Shortcut::Toggle,
            Shortcut::SelectAll,
            Shortcut::DeselectAll,
            Shortcut::Delete,
            Shortcut::Sort,
            Shortcut::OpenFolder,
        ],
        AppState::ConfirmingDeletion => vec![
            Shortcut::Confirm,
            Shortcut::Cancel,
        ],
        AppState::Deleting => vec![
            Shortcut::Quit, // Force quit only
        ],
        AppState::Error => vec![
            Shortcut::Quit,
            Shortcut::Cancel, // Return to browsing
        ],
        AppState::Help => vec![
            // Any key returns to browsing
        ],
        AppState::Quit => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_shortcut_matching() {
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(Shortcut::Quit.matches_shortcut(&quit_key));

        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(Shortcut::Quit.matches_shortcut(&esc_key));

        let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(Shortcut::SelectAll.matches_shortcut(&ctrl_a));

        let regular_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!Shortcut::SelectAll.matches_shortcut(&regular_a));
    }

    #[test]
    fn test_shortcut_display_strings() {
        assert_eq!(Shortcut::Quit.display_string(), "q/Esc");
        assert_eq!(Shortcut::Help.display_string(), "h/F1");
        assert_eq!(Shortcut::SelectAll.display_string(), "Ctrl+A");
    }

    #[test]
    fn test_shortcuts_for_state() {
        let loading_shortcuts = get_shortcuts_for_state(&crate::tui::AppState::Loading);
        assert_eq!(loading_shortcuts.len(), 1);
        assert!(loading_shortcuts.contains(&Shortcut::Quit));

        let browsing_shortcuts = get_shortcuts_for_state(&crate::tui::AppState::Browsing);
        assert!(browsing_shortcuts.len() > 5);
        assert!(browsing_shortcuts.contains(&Shortcut::Quit));
        assert!(browsing_shortcuts.contains(&Shortcut::Help));
        assert!(browsing_shortcuts.contains(&Shortcut::Delete));
    }

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new(Duration::from_millis(100));
        assert!(handler.is_ok());
    }
}
