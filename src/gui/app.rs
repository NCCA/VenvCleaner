//! GUI Application State Management
//!
//! This module handles the application state for the GUI mode, including
//! .venv directory management, selection state, sorting, and user interactions.
//! It implements the eframe::App trait for the main GUI loop.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use eframe::egui::{self, *};
use log::{debug, info, warn};

use crate::core::{VenvCleaner, VenvInfo, Result};
use super::{GuiAppState, GuiSortBy, GuiEvent, utils};

/// Main GUI application state
pub struct GuiApp {
    /// Current application state
    state: GuiAppState,
    /// The core VenvCleaner instance
    cleaner: VenvCleaner,
    /// List of found .venv directories
    venvs: Vec<VenvInfo>,
    /// Currently selected indices in the list
    selected_venvs: HashSet<usize>,
    /// Current sorting method
    sort_by: GuiSortBy,
    /// Reverse sort order
    reverse_sort: bool,
    /// Current status message
    status: String,
    /// Error message (if any)
    error_message: String,
    /// Current directory being browsed
    current_directory: PathBuf,
    /// Whether search is recursive
    is_recursive: bool,
    /// Loading animation state
    loading_dots: usize,
    /// Last animation update time
    last_animation_update: Instant,
    /// Channel for background tasks
    event_receiver: Option<Receiver<GuiEvent>>,
    event_sender: Option<Sender<GuiEvent>>,
    /// Deletion progress
    deletion_progress: f32,
    /// Whether to show confirmation dialog
    show_confirmation_dialog: bool,
    /// Whether to show help window
    show_help: bool,
    /// Whether to show about window
    show_about: bool,
    /// Whether to show folder selection dialog
    show_folder_dialog: bool,
    /// New directory path from folder dialog
    pending_directory: Option<PathBuf>,
    /// Search filter text
    search_filter: String,
    /// Whether to use dark theme
    dark_theme: bool,
    /// Window sizes and positions
    main_window_size: Vec2,
    /// Table scroll position
    table_scroll: f32,
}

impl GuiApp {
    /// Create a new GUI application instance
    pub fn new(
        cleaner: VenvCleaner,
        base_directory: PathBuf,
        recursive: bool,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut app = Self {
            state: GuiAppState::Loading,
            cleaner,
            venvs: Vec::new(),
            selected_venvs: HashSet::new(),
            sort_by: GuiSortBy::Path,
            reverse_sort: false,
            status: "Initializing VenvCleaner...".to_string(),
            error_message: String::new(),
            current_directory: base_directory,
            is_recursive: recursive,
            loading_dots: 0,
            last_animation_update: Instant::now(),
            event_receiver: Some(receiver),
            event_sender: Some(sender),
            deletion_progress: 0.0,
            show_confirmation_dialog: false,
            show_help: false,
            show_about: false,
            show_folder_dialog: false,
            pending_directory: None,
            search_filter: String::new(),
            dark_theme: false,
            main_window_size: Vec2::new(1200.0, 800.0),
            table_scroll: 0.0,
        };

        // Start loading .venv directories immediately
        app.start_loading_venvs();
        app
    }

    /// Start loading .venv directories in background
    fn start_loading_venvs(&mut self) {
        if let Some(sender) = &self.event_sender {
            let cleaner = VenvCleaner::new(
                self.current_directory.clone(),
                self.is_recursive,
                false,
                false,
                0,
            );
            let sender_clone = sender.clone();

            self.state = GuiAppState::Loading;
            let search_mode = if self.is_recursive { "recursively" } else { "in current directory" };
            self.status = format!("🔍 Scanning for .venv directories {}...", search_mode);

            thread::spawn(move || {
                debug!("Starting .venv loading task in background");
                match cleaner.find_venv_directories() {
                    Ok(venvs) => {
                        debug!("Found {} .venv directories", venvs.len());
                        let _ = sender_clone.send(GuiEvent::VenvsLoaded(venvs));
                    }
                    Err(e) => {
                        warn!("Error loading .venv directories: {}", e);
                        let _ = sender_clone.send(GuiEvent::LoadError(e.to_string()));
                    }
                }
            });
        }
    }

    /// Start deletion of selected .venv directories
    fn start_deletion(&mut self) {
        if let Some(sender) = &self.event_sender {
            let selected_venvs: Vec<VenvInfo> = self.selected_venvs
                .iter()
                .filter_map(|&i| self.venvs.get(i))
                .cloned()
                .collect();

            if selected_venvs.is_empty() {
                return;
            }

            let cleaner = VenvCleaner::new(
                self.current_directory.clone(),
                self.is_recursive,
                false,
                false,
                0,
            );
            let sender_clone = sender.clone();

            self.state = GuiAppState::Deleting;
            self.deletion_progress = 0.0;
            self.status = format!("Deleting {} directories...", selected_venvs.len());

            thread::spawn(move || {
                debug!("Starting deletion task for {} directories", selected_venvs.len());
                let mut results = Vec::new();

                for venv in selected_venvs {
                    let result = cleaner.delete_venv_directory(&venv);
                    results.push((venv, result));
                }

                debug!("Deletion task completed");
                let _ = sender_clone.send(GuiEvent::DeletionComplete(results));
            });
        }
    }

    /// Handle background events
    fn handle_events(&mut self) {
        let mut events = Vec::new();
        if let Some(receiver) = &self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                GuiEvent::VenvsLoaded(venvs) => {
                    self.venvs = venvs;
                    self.sort_venvs();
                    self.state = GuiAppState::Browsing;
                    self.selected_venvs.clear();

                    if self.venvs.is_empty() {
                        self.status = "No .venv directories found. Try changing the search directory or enabling recursive search.".to_string();
                    } else {
                        self.status = format!("Found {} .venv directories. Select directories to delete or use the search filter.", self.venvs.len());
                    }
                }
                GuiEvent::LoadError(error) => {
                    self.error_message = error;
                    self.state = GuiAppState::Error;
                }
                GuiEvent::DeletionComplete(results) => {
                    self.handle_deletion_results(results);
                    // Refresh the list after deletion
                    self.start_loading_venvs();
                }
            }
        }
    }

    /// Handle deletion results
    fn handle_deletion_results(&mut self, results: Vec<(VenvInfo, Result<()>)>) {
        let successful = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed = results.len() - successful;

        if failed == 0 {
            self.status = format!("✅ Successfully deleted {} directories. List will refresh automatically.", successful);
        } else {
            self.status = format!("⚠️ Deleted {} directories, {} failed. Check permissions for failed items.", successful, failed);
        }

        self.selected_venvs.clear();
        self.show_confirmation_dialog = false;
        self.state = GuiAppState::Loading; // Will transition to Browsing when refresh completes
    }

    /// Sort the current list of venvs
    fn sort_venvs(&mut self) {
        match self.sort_by {
            GuiSortBy::Path => {
                self.venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        b.path().cmp(a.path())
                    } else {
                        a.path().cmp(b.path())
                    }
                });
            }
            GuiSortBy::Size => {
                self.venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.size_bytes().cmp(&b.size_bytes())
                    } else {
                        b.size_bytes().cmp(&a.size_bytes())
                    }
                });
            }
            GuiSortBy::Created => {
                self.venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.created().cmp(b.created())
                    } else {
                        b.created().cmp(a.created())
                    }
                });
            }
            GuiSortBy::LastModified => {
                self.venvs.sort_by(|a, b| {
                    if self.reverse_sort {
                        a.last_modified().cmp(b.last_modified())
                    } else {
                        b.last_modified().cmp(a.last_modified())
                    }
                });
            }
        }
    }

    /// Get filtered venvs based on search filter
    fn get_filtered_venvs(&self) -> Vec<(usize, &VenvInfo)> {
        if self.search_filter.is_empty() {
            self.venvs.iter().enumerate().collect()
        } else {
            self.venvs
                .iter()
                .enumerate()
                .filter(|(_, venv)| {
                    let search_lower = self.search_filter.to_lowercase();
                    venv.location().to_lowercase().contains(&search_lower) ||
                    venv.path().display().to_string().to_lowercase().contains(&search_lower)
                })
                .collect()
        }
    }

    /// Update loading animation
    fn update_animation(&mut self) {
        if self.last_animation_update.elapsed() >= std::time::Duration::from_millis(500) {
            self.loading_dots = (self.loading_dots + 1) % 4;
            self.last_animation_update = Instant::now();
        }
    }

    /// Draw the main menu bar
    fn draw_menu_bar(&mut self, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.start_loading_venvs();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("📁 Change Directory").clicked() {
                    self.show_folder_dialog = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("❌ Quit").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Select All").clicked() {
                    if !self.venvs.is_empty() {
                        self.selected_venvs = (0..self.venvs.len()).collect();
                    }
                    ui.close_menu();
                }
                if ui.button("Select None").clicked() {
                    self.selected_venvs.clear();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🗑️ Delete Selected").clicked() {
                    if !self.selected_venvs.is_empty() {
                        self.show_confirmation_dialog = true;
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                ui.menu_button(format!("Sort by: {}", self.sort_by.display_name()), |ui| {
                    if ui.selectable_label(matches!(self.sort_by, GuiSortBy::Path), "Path").clicked() {
                        self.sort_by = GuiSortBy::Path;
                        self.sort_venvs();
                        ui.close_menu();
                    }
                    if ui.selectable_label(matches!(self.sort_by, GuiSortBy::Size), "Size").clicked() {
                        self.sort_by = GuiSortBy::Size;
                        self.sort_venvs();
                        ui.close_menu();
                    }
                    if ui.selectable_label(matches!(self.sort_by, GuiSortBy::Created), "Created").clicked() {
                        self.sort_by = GuiSortBy::Created;
                        self.sort_venvs();
                        ui.close_menu();
                    }
                    if ui.selectable_label(matches!(self.sort_by, GuiSortBy::LastModified), "Last Used").clicked() {
                        self.sort_by = GuiSortBy::LastModified;
                        self.sort_venvs();
                        ui.close_menu();
                    }
                });

                if ui.checkbox(&mut self.reverse_sort, "Reverse Sort").clicked() {
                    self.sort_venvs();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("📖 Help").clicked() {
                    self.show_help = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("ℹ️ About").clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    /// Draw the toolbar
    fn draw_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Refresh button
            if ui.button("🔄 Refresh").clicked() {
                self.start_loading_venvs();
            }

            ui.separator();

            // Sort controls
            ui.label("Sort:");
            ComboBox::from_id_source("sort_combo")
                .selected_text(self.sort_by.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, GuiSortBy::Path, "Path");
                    ui.selectable_value(&mut self.sort_by, GuiSortBy::Size, "Size");
                    ui.selectable_value(&mut self.sort_by, GuiSortBy::Created, "Created");
                    ui.selectable_value(&mut self.sort_by, GuiSortBy::LastModified, "Last Used");
                });

            if ui.button(if self.reverse_sort { "🔽" } else { "🔼" }).clicked() {
                self.reverse_sort = !self.reverse_sort;
                self.sort_venvs();
            }

            ui.separator();

            // Selection controls
            if ui.button("Select All").clicked() {
                if !self.venvs.is_empty() {
                    self.selected_venvs = (0..self.venvs.len()).collect();
                }
            }

            if ui.button("Select None").clicked() {
                self.selected_venvs.clear();
            }

            ui.separator();

            // Delete button
            ui.add_enabled_ui(!self.selected_venvs.is_empty(), |ui| {
                if ui.button(format!("🗑️ Delete Selected ({})", self.selected_venvs.len())).clicked() {
                    self.show_confirmation_dialog = true;
                }
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Search filter
                ui.add_sized([200.0, 20.0], TextEdit::singleline(&mut self.search_filter).hint_text("Search directories..."));
                ui.label("🔍");
            });
        });
    }

    /// Draw the main content area
    fn draw_main_content(&mut self, ui: &mut Ui) {
        match self.state {
            GuiAppState::Loading => self.draw_loading_screen(ui),
            GuiAppState::Browsing => self.draw_venv_list(ui),
            GuiAppState::Deleting => self.draw_deletion_progress(ui),
            GuiAppState::Error => self.draw_error_screen(ui),
            _ => {}
        }
    }

    /// Draw the loading screen
    fn draw_loading_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            ui.heading("VenvCleaner");
            ui.add_space(20.0);

            let loading_text = format!("Loading{}", ".".repeat(self.loading_dots));
            ui.label(RichText::new(loading_text).size(18.0));

            ui.add_space(20.0);

            ui.label(format!("Directory: {}", self.current_directory.display()));
            ui.label(format!("Mode: {}", if self.is_recursive { "Recursive search" } else { "Current directory only" }));

            ui.add_space(20.0);
            ui.label(&self.status);

            ui.add_space(40.0);
            ui.label("Please wait while scanning directories...");
        });
    }

    /// Draw the .venv directory list
    fn draw_venv_list(&mut self, ui: &mut Ui) {
        // Collect filtered data first to avoid borrowing issues
        let filtered_indices: Vec<usize> = if self.search_filter.is_empty() {
            (0..self.venvs.len()).collect()
        } else {
            let search_lower = self.search_filter.to_lowercase();
            self.venvs
                .iter()
                .enumerate()
                .filter(|(_, venv)| {
                    venv.location().to_lowercase().contains(&search_lower) ||
                    venv.path().display().to_string().to_lowercase().contains(&search_lower)
                })
                .map(|(i, _)| i)
                .collect()
        };

        // Summary info
        ui.horizontal(|ui| {
            ui.label(format!("Found {} directories", self.venvs.len()));
            if !self.search_filter.is_empty() {
                ui.label(format!("(showing {} filtered)", filtered_indices.len()));
            }
            if !self.selected_venvs.is_empty() {
                let total_size: u64 = self.selected_venvs
                    .iter()
                    .filter_map(|&i| self.venvs.get(i))
                    .map(|v| v.size_bytes())
                    .sum();
                ui.label(format!("| Selected: {} ({} total)",
                    self.selected_venvs.len(),
                    utils::format_size(total_size)));
            }
        });

        ui.separator();

        // Table header
        ScrollArea::vertical()
            .id_source("venv_table")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + 14.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            // Column headers
                            if ui.selectable_label(false, "Select").clicked() {
                                // Toggle all selection
                                if self.selected_venvs.len() == filtered_indices.len() {
                                    self.selected_venvs.clear();
                                } else {
                                    self.selected_venvs = filtered_indices.iter().cloned().collect();
                                }
                            }
                            ui.separator();

                            ui.allocate_ui_with_layout(
                                Vec2::new(40.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| { ui.label("Age"); },
                            );
                            ui.separator();

                            ui.allocate_ui_with_layout(
                                Vec2::new(400.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| { ui.label("Location"); },
                            );
                            ui.separator();

                            ui.allocate_ui_with_layout(
                                Vec2::new(100.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| { ui.label("Size"); },
                            );
                            ui.separator();

                            ui.allocate_ui_with_layout(
                                Vec2::new(150.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| { ui.label("Last Used"); },
                            );
                            ui.separator();

                            ui.allocate_ui_with_layout(
                                Vec2::new(100.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| { ui.label("Actions"); },
                            );
                        },
                    );
                });

                ui.separator();

                // Table rows
                for &original_index in &filtered_indices {
                    if let Some(venv) = self.venvs.get(original_index) {
                        let is_selected = self.selected_venvs.contains(&original_index);

                        let row_response = ui.horizontal(|ui| {
                            // Selection checkbox
                            let mut selected = is_selected;
                            if ui.checkbox(&mut selected, "").clicked() {
                                if selected {
                                    self.selected_venvs.insert(original_index);
                                } else {
                                    self.selected_venvs.remove(&original_index);
                                }
                            }
                            ui.separator();

                            // Age indicator
                            let age_days = venv.age_in_days();
                            ui.allocate_ui_with_layout(
                                Vec2::new(40.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.colored_label(
                                        utils::get_age_color(age_days),
                                        format!("{} {}d", utils::get_age_indicator(age_days), age_days)
                                    );
                                },
                            );
                            ui.separator();

                            // Location
                            ui.allocate_ui_with_layout(
                                Vec2::new(400.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.label(utils::format_path_for_display(&venv.location(), 60));
                                },
                            );
                            ui.separator();

                            // Size
                            ui.allocate_ui_with_layout(
                                Vec2::new(100.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.colored_label(
                                        utils::get_size_color(venv.size_bytes()),
                                        utils::format_size(venv.size_bytes())
                                    );
                                },
                            );
                            ui.separator();

                            // Last used
                            ui.allocate_ui_with_layout(
                                Vec2::new(150.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.label(venv.last_modified_formatted());
                                },
                            );
                            ui.separator();

                            // Actions
                            ui.allocate_ui_with_layout(
                                Vec2::new(100.0, 20.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    if ui.small_button("📁 Open").clicked() {
                                        if let Some(parent) = venv.parent_path() {
                                            let _ = open::that(parent);
                                        }
                                    }
                                },
                            );
                        });

                        // Row selection on click
                        if row_response.response.clicked() {
                            if self.selected_venvs.contains(&original_index) {
                                self.selected_venvs.remove(&original_index);
                            } else {
                                self.selected_venvs.insert(original_index);
                            }
                        }

                        // Highlight selected rows
                        if is_selected {
                            let rect = row_response.response.rect;
                            ui.painter().rect_filled(
                                rect,
                                Rounding::same(2.0),
                                Color32::from_rgba_unmultiplied(100, 150, 255, 30)
                            );
                        }
                    }
                }
            });
    }

    /// Draw deletion progress
    fn draw_deletion_progress(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Deleting Directories");
            ui.add_space(20.0);

            ui.label(&self.status);
            ui.add_space(20.0);

            let progress_bar = ProgressBar::new(self.deletion_progress)
                .text(format!("{:.0}%", self.deletion_progress * 100.0));
            ui.add(progress_bar);

            ui.add_space(20.0);
            ui.label("Please wait...");
        });
    }

    /// Draw error screen
    fn draw_error_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Error");
            ui.add_space(20.0);

            ui.colored_label(Color32::RED, &self.error_message);
            ui.add_space(20.0);

            if ui.button("Try Again").clicked() {
                self.start_loading_venvs();
            }
        });
    }

    /// Draw status bar
    fn draw_status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(&self.status);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("Directory: {} ({})",
                    self.current_directory.display(),
                    if self.is_recursive { "Recursive" } else { "Current only" }
                ));
            });
        });
    }

    /// Draw confirmation dialog
    fn draw_confirmation_dialog(&mut self, ctx: &Context) {
        if !self.show_confirmation_dialog {
            return;
        }

        Window::new("Confirm Deletion")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("⚠️ Confirm Deletion");
                    ui.add_space(20.0);

                    let selected_venvs: Vec<&VenvInfo> = self.selected_venvs
                        .iter()
                        .filter_map(|&i| self.venvs.get(i))
                        .collect();

                    let total_size: u64 = selected_venvs.iter().map(|v| v.size_bytes()).sum();

                    ui.label(format!("You are about to permanently delete {} .venv directories.", selected_venvs.len()));
                    ui.add_space(10.0);
                    ui.label(format!("Total size to be freed: {}", utils::format_size(total_size)));
                    ui.add_space(10.0);
                    ui.colored_label(Color32::RED, "⚠️ This action cannot be undone!");
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("❌ Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }

                        ui.add_space(20.0);

                        if ui.button("🗑️ Delete").clicked() {
                            self.show_confirmation_dialog = false;
                            self.start_deletion();
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }

    /// Draw help window
    fn draw_help_window(&mut self, ctx: &Context) {
        if !self.show_help {
            return;
        }

        Window::new("Help")
            .collapsible(true)
            .resizable(true)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("VenvCleaner GUI Help");
                    ui.separator();

                    ui.heading("Overview");
                    ui.label("VenvCleaner helps you find, analyze, and clean up Python virtual environment folders (.venv) on your system.");
                    ui.add_space(10.0);

                    ui.heading("Usage");
                    ui.label("• Use the table to view all .venv directories found");
                    ui.label("• Click checkboxes or rows to select directories for deletion");
                    ui.label("• Use the search box to filter directories");
                    ui.label("• Sort by different criteria using the dropdown");
                    ui.label("• Click 'Delete Selected' to remove chosen directories");
                    ui.add_space(10.0);

                    ui.heading("Color Coding");
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(100, 255, 100), "🟢");
                        ui.label("Recently used (<30 days)");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(255, 255, 100), "🟡");
                        ui.label("Moderately used (30-90 days)");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), "🔴");
                        ui.label("Old (>90 days)");
                    });
                    ui.add_space(10.0);

                    ui.heading("Keyboard Shortcuts");
                    ui.label("• Ctrl+A: Select all directories");
                    ui.label("• Delete: Delete selected directories");
                    ui.label("• F5: Refresh list");
                    ui.add_space(10.0);

                    if ui.button("Close").clicked() {
                        self.show_help = false;
                    }
                });
            });
    }

    /// Draw about window
    fn draw_about_window(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }

        Window::new("About VenvCleaner")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("VenvCleaner");
                    ui.add_space(10.0);
                    ui.label("Version 0.1.0");
                    ui.add_space(20.0);
                    ui.label("A multi-mode application to help manage and clean up");
                    ui.label("Python virtual environment folders (.venv) on Mac and Linux.");
                    ui.add_space(20.0);
                    ui.label("Built with Rust and egui");
                    ui.add_space(20.0);

                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                    ui.add_space(10.0);
                });
            });
    }

    /// Draw folder selection dialog
    fn draw_folder_dialog(&mut self, ctx: &Context) {
        if !self.show_folder_dialog {
            return;
        }

        Window::new("Select Directory")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    ui.label("Current directory:");
                    ui.label(RichText::new(self.current_directory.display().to_string()).monospace());
                    ui.add_space(10.0);

                    ui.label("Enter new directory path:");
                    let mut path_text = self.pending_directory
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| self.current_directory.display().to_string());

                    if ui.text_edit_singleline(&mut path_text).changed() {
                        self.pending_directory = Some(PathBuf::from(path_text));
                    }

                    ui.add_space(10.0);

                    // Common shortcuts for easy access
                    ui.label("Quick shortcuts:");
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("🏠 Home").clicked() {
                            if let Some(home) = dirs::home_dir() {
                                self.pending_directory = Some(home);
                            }
                        }
                        if ui.button("📁 Documents").clicked() {
                            if let Some(docs) = dirs::document_dir() {
                                self.pending_directory = Some(docs);
                            }
                        }
                        if ui.button("💻 Desktop").clicked() {
                            if let Some(desktop) = dirs::desktop_dir() {
                                self.pending_directory = Some(desktop);
                            }
                        }
                        if ui.button("📂 Downloads").clicked() {
                            if let Some(downloads) = dirs::download_dir() {
                                self.pending_directory = Some(downloads);
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Browse...").clicked() {
                            // Try to open native file dialog
                            #[cfg(feature = "gui")]
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&self.current_directory)
                                    .pick_folder()
                                {
                                    self.pending_directory = Some(path);
                                }
                            }
                            #[cfg(not(feature = "gui"))]
                            {
                                // Fallback for when rfd is not available
                                self.status = "Native file dialog not available. Please enter path manually.".to_string();
                            }
                        }

                        ui.add_space(10.0);

                        if ui.button("Cancel").clicked() {
                            self.show_folder_dialog = false;
                            self.pending_directory = None;
                        }

                        ui.add_space(10.0);

                        let pending_path = self.pending_directory.as_ref().unwrap_or(&self.current_directory);
                        let is_valid = pending_path.exists() && pending_path.is_dir();

                        ui.add_enabled_ui(is_valid, |ui| {
                            if ui.button("Select").clicked() {
                                if let Some(new_path) = self.pending_directory.clone() {
                                    if new_path.exists() && new_path.is_dir() {
                                        self.current_directory = new_path.clone();
                                        self.is_recursive = true; // Keep recursive for GUI
                                        self.show_folder_dialog = false;
                                        self.pending_directory = None;
                                        self.status = format!("Changed directory to: {}", new_path.display());
                                        self.start_loading_venvs();
                                    }
                                }
                            }
                        });
                    });

                    if let Some(pending_path) = &self.pending_directory {
                        ui.add_space(10.0);
                        if !pending_path.exists() {
                            ui.colored_label(Color32::RED, "⚠️ Directory does not exist");
                        } else if !pending_path.is_dir() {
                            ui.colored_label(Color32::RED, "⚠️ Path is not a directory");
                        } else {
                            ui.colored_label(Color32::GREEN, "✅ Valid directory");
                        }
                    }

                    ui.add_space(10.0);
                });
            });
    }
}

impl eframe::App for GuiApp {
    /// Update the application
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Handle background events
        self.handle_events();

        // Update animations
        self.update_animation();

        // Request repaint for animations
        if matches!(self.state, GuiAppState::Loading | GuiAppState::Deleting) {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(Key::F5) {
                self.start_loading_venvs();
            }
            if i.modifiers.ctrl && i.key_pressed(Key::A) {
                if !self.venvs.is_empty() {
                    self.selected_venvs = (0..self.venvs.len()).collect();
                }
            }
            if i.key_pressed(Key::Delete) && !self.selected_venvs.is_empty() {
                self.show_confirmation_dialog = true;
            }
            if i.key_pressed(Key::Escape) {
                self.show_confirmation_dialog = false;
                self.show_help = false;
                self.show_about = false;
            }
        });

        // Main UI
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Menu bar
                self.draw_menu_bar(ui);
                ui.separator();

                // Toolbar
                self.draw_toolbar(ui);
                ui.separator();

                // Main content
                self.draw_main_content(ui);

                // Status bar
                ui.separator();
                self.draw_status_bar(ui);
            });
        });

        // Modal dialogs
        self.draw_confirmation_dialog(ctx);
        self.draw_help_window(ctx);
        self.draw_about_window(ctx);
        self.draw_folder_dialog(ctx);
    }

    /// Save application state
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "search_filter", &self.search_filter);
        eframe::set_value(storage, "sort_by", &(self.sort_by as u8));
        eframe::set_value(storage, "reverse_sort", &self.reverse_sort);
    }

    /// Auto-save interval
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }
}
