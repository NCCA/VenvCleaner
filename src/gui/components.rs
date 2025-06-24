//! GUI Components module for reusable UI elements
//!
//! This module provides reusable UI components for the VenvCleaner GUI,
//! including custom widgets, dialogs, and specialized controls.

use eframe::egui::{self, *};
use crate::core::VenvInfo;
use super::utils;

/// A custom table widget for displaying .venv directories
pub struct VenvTable<'a> {
    venvs: &'a [VenvInfo],
    selected: &'a mut std::collections::HashSet<usize>,
    search_filter: &'a str,
}

impl<'a> VenvTable<'a> {
    /// Create a new VenvTable
    pub fn new(
        venvs: &'a [VenvInfo],
        selected: &'a mut std::collections::HashSet<usize>,
        search_filter: &'a str,
    ) -> Self {
        Self {
            venvs,
            selected,
            search_filter,
        }
    }

    /// Show the table
    pub fn show(self, ui: &mut Ui) {
        let available_width = ui.available_width();

        // Calculate column widths
        let checkbox_width = 30.0;
        let age_width = 60.0;
        let size_width = 100.0;
        let date_width = 150.0;
        let actions_width = 80.0;
        let location_width = available_width - checkbox_width - age_width - size_width - date_width - actions_width - 50.0;
        let row_height = 24.0;

        ScrollArea::vertical()
            .id_source("venv_table_scroll")
            .show(ui, |ui| {
                // Table header
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(checkbox_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let all_selected = self.selected.len() == self.venvs.len() && !self.venvs.is_empty();
                            let mut select_all = all_selected;
                            if ui.checkbox(&mut select_all, "").changed() {
                                if select_all {
                                    *self.selected = (0..self.venvs.len()).collect();
                                } else {
                                    self.selected.clear();
                                }
                            }
                        },
                    );

                    ui.separator();

                    ui.allocate_ui_with_layout(
                        Vec2::new(age_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.strong("Age");
                        },
                    );

                    ui.separator();

                    ui.allocate_ui_with_layout(
                        Vec2::new(location_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.strong("Location");
                        },
                    );

                    ui.separator();

                    ui.allocate_ui_with_layout(
                        Vec2::new(size_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.strong("Size");
                        },
                    );

                    ui.separator();

                    ui.allocate_ui_with_layout(
                        Vec2::new(date_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.strong("Last Used");
                        },
                    );

                    ui.separator();

                    ui.allocate_ui_with_layout(
                        Vec2::new(actions_width, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.strong("Actions");
                        },
                    );
                });

                ui.separator();

                // Filter venvs based on search
                let filtered_venvs: Vec<(usize, &VenvInfo)> = self.venvs
                    .iter()
                    .enumerate()
                    .filter(|(_, venv)| {
                        if self.search_filter.is_empty() {
                            true
                        } else {
                            let search_lower = self.search_filter.to_lowercase();
                            venv.location().to_lowercase().contains(&search_lower) ||
                            venv.path().display().to_string().to_lowercase().contains(&search_lower)
                        }
                    })
                    .collect();

                // Table rows
                for (original_index, venv) in filtered_venvs {
                    let is_selected = self.selected.contains(&original_index);

                    let row_response = ui.horizontal(|ui| {
                        // Selection checkbox
                        ui.allocate_ui_with_layout(
                            Vec2::new(checkbox_width, row_height),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let mut selected = is_selected;
                                if ui.checkbox(&mut selected, "").clicked() {
                                    if selected {
                                        self.selected.insert(original_index);
                                    } else {
                                        self.selected.remove(&original_index);
                                    }
                                }
                            },
                        );

                        ui.separator();

                        // Age indicator
                        ui.allocate_ui_with_layout(
                            Vec2::new(age_width, row_height),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let age_days = venv.age_in_days();
                                ui.colored_label(
                                    utils::get_age_color(age_days),
                                    format!("{} {}d", utils::get_age_indicator(age_days), age_days)
                                );
                            },
                        );

                        ui.separator();

                        // Location
                        ui.allocate_ui_with_layout(
                            Vec2::new(location_width, row_height),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.label(utils::format_path_for_display(&venv.location(), 60));
                            },
                        );

                        ui.separator();

                        // Size
                        ui.allocate_ui_with_layout(
                            Vec2::new(size_width, row_height),
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
                            Vec2::new(date_width, row_height),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.label(venv.last_modified_formatted());
                            },
                        );

                        ui.separator();

                        // Actions
                        ui.allocate_ui_with_layout(
                            Vec2::new(actions_width, row_height),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                if ui.small_button("📁").on_hover_text("Open folder").clicked() {
                                    if let Some(parent) = venv.parent_path() {
                                        let _ = open::that(parent);
                                    }
                                }
                            },
                        );
                    });

                    // Row selection on click
                    if row_response.response.clicked() {
                        if self.selected.contains(&original_index) {
                            self.selected.remove(&original_index);
                        } else {
                            self.selected.insert(original_index);
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

                    // Hover effect
                    if row_response.response.hovered() && !is_selected {
                        let rect = row_response.response.rect;
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(2.0),
                            Color32::from_rgba_unmultiplied(100, 150, 255, 15)
                        );
                    }
                }

                ui.allocate_space(Vec2::new(ui.available_width(), 10.0));
            });
    }
}

/// A status card widget for displaying summary information
pub struct StatusCard {
    title: String,
    value: String,
    icon: String,
    color: Color32,
}

impl StatusCard {
    /// Create a new status card
    pub fn new(title: impl Into<String>, value: impl Into<String>, icon: impl Into<String>, color: Color32) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            icon: icon.into(),
            color,
        }
    }

    /// Show the status card
    pub fn show(self, ui: &mut Ui) -> Response {
        Frame::none()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 20)))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(self.color, RichText::new(self.icon).size(24.0));
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.title).size(12.0).color(Color32::GRAY));
                        ui.label(RichText::new(self.value).size(16.0).strong());
                    });
                });
            })
            .response
    }
}

/// A confirmation dialog component
pub struct ConfirmationDialog<'a> {
    title: &'a str,
    message: &'a str,
    confirm_text: &'a str,
    cancel_text: &'a str,
    danger: bool,
}

impl<'a> ConfirmationDialog<'a> {
    /// Create a new confirmation dialog
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            confirm_text: "OK",
            cancel_text: "Cancel",
            danger: false,
        }
    }

    /// Set custom button text
    pub fn buttons(mut self, confirm: &'a str, cancel: &'a str) -> Self {
        self.confirm_text = confirm;
        self.cancel_text = cancel;
        self
    }

    /// Mark as a dangerous action (red confirm button)
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Show the dialog and return the user's choice
    pub fn show(self, ctx: &Context) -> DialogResult {
        let mut result = DialogResult::None;

        Window::new(self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(self.message);
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button(self.cancel_text).clicked() {
                            result = DialogResult::Cancel;
                        }

                        ui.add_space(20.0);

                        let confirm_button = if self.danger {
                            Button::new(self.confirm_text).fill(Color32::from_rgb(220, 53, 69))
                        } else {
                            Button::new(self.confirm_text)
                        };

                        if ui.add(confirm_button).clicked() {
                            result = DialogResult::Confirm;
                        }
                    });

                    ui.add_space(10.0);
                });
            });

        result
    }
}

/// Result of a dialog interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    None,
    Confirm,
    Cancel,
}

/// A progress indicator component
pub struct ProgressIndicator {
    progress: f32,
    text: Option<String>,
    show_percentage: bool,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            text: None,
            show_percentage: true,
        }
    }

    /// Set custom text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Hide percentage display
    pub fn hide_percentage(mut self) -> Self {
        self.show_percentage = false;
        self
    }

    /// Show the progress indicator
    pub fn show(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width().min(300.0), 20.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let _visuals = ui.style().interact(&response);

            // Background
            ui.painter().rect_filled(
                rect,
                Rounding::same(4.0),
                Color32::from_rgba_unmultiplied(200, 200, 200, 100)
            );

            // Progress bar
            let progress_width = rect.width() * self.progress;
            let progress_rect = Rect::from_min_size(
                rect.min,
                Vec2::new(progress_width, rect.height())
            );

            let progress_color = if self.progress >= 1.0 {
                Color32::from_rgb(40, 180, 40) // Green when complete
            } else {
                Color32::from_rgb(70, 130, 200) // Blue for progress
            };

            ui.painter().rect_filled(
                progress_rect,
                Rounding::same(4.0),
                progress_color
            );

            // Text overlay
            let text = if let Some(ref custom_text) = self.text {
                custom_text.clone()
            } else if self.show_percentage {
                format!("{:.1}%", self.progress * 100.0)
            } else {
                String::new()
            };

            if !text.is_empty() {
                let text_color = if self.progress > 0.5 {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };

                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    text,
                    FontId::default(),
                    text_color
                );
            }
        }

        response
    }
}

/// A toolbar component with common actions
pub struct Toolbar<'a> {
    actions: Vec<ToolbarAction<'a>>,
}

impl<'a> Toolbar<'a> {
    /// Create a new toolbar
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add an action to the toolbar
    pub fn action(mut self, action: ToolbarAction<'a>) -> Self {
        self.actions.push(action);
        self
    }

    /// Show the toolbar
    pub fn show(self, ui: &mut Ui) -> Vec<bool> {
        let mut results = Vec::new();

        ui.horizontal(|ui| {
            for action in self.actions {
                let clicked = match action {
                    ToolbarAction::Button { text, icon, enabled, .. } => {
                        let button_text = if let Some(icon) = icon {
                            format!("{} {}", icon, text)
                        } else {
                            text.to_string()
                        };

                        ui.add_enabled(enabled, Button::new(button_text)).clicked()
                    }
                    ToolbarAction::Separator => {
                        ui.separator();
                        false
                    }
                };
                results.push(clicked);
            }
        });

        results
    }
}

impl<'a> Default for Toolbar<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// An action in a toolbar
pub enum ToolbarAction<'a> {
    Button {
        text: &'a str,
        icon: Option<&'a str>,
        enabled: bool,
        tooltip: Option<&'a str>,
    },
    Separator,
}

impl<'a> ToolbarAction<'a> {
    /// Create a button action
    pub fn button(text: &'a str) -> Self {
        Self::Button {
            text,
            icon: None,
            enabled: true,
            tooltip: None,
        }
    }

    /// Add an icon to the button
    pub fn icon(mut self, icon: &'a str) -> Self {
        if let Self::Button { icon: ref mut i, .. } = self {
            *i = Some(icon);
        }
        self
    }

    /// Set button enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        if let Self::Button { enabled: ref mut e, .. } = self {
            *e = enabled;
        }
        self
    }

    /// Add a tooltip to the button
    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        if let Self::Button { tooltip: ref mut t, .. } = self {
            *t = Some(tooltip);
        }
        self
    }

    /// Create a separator
    pub fn separator() -> Self {
        Self::Separator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::VenvInfo;
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
    fn test_status_card_creation() {
        let card = StatusCard::new("Test", "123", "🔍", Color32::BLUE);
        assert_eq!(card.title, "Test");
        assert_eq!(card.value, "123");
        assert_eq!(card.icon, "🔍");
        assert_eq!(card.color, Color32::BLUE);
    }

    #[test]
    fn test_confirmation_dialog_creation() {
        let dialog = ConfirmationDialog::new("Test Title", "Test Message")
            .buttons("Yes", "No")
            .danger();

        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test Message");
        assert_eq!(dialog.confirm_text, "Yes");
        assert_eq!(dialog.cancel_text, "No");
        assert!(dialog.danger);
    }

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator::new(0.75)
            .text("Loading...")
            .hide_percentage();

        assert_eq!(progress.progress, 0.75);
        assert_eq!(progress.text, Some("Loading...".to_string()));
        assert!(!progress.show_percentage);
    }

    #[test]
    fn test_progress_indicator_clamping() {
        let progress_low = ProgressIndicator::new(-0.5);
        assert_eq!(progress_low.progress, 0.0);

        let progress_high = ProgressIndicator::new(1.5);
        assert_eq!(progress_high.progress, 1.0);
    }

    #[test]
    fn test_toolbar_creation() {
        let toolbar = Toolbar::new()
            .action(ToolbarAction::button("Test").icon("🔍").enabled(false))
            .action(ToolbarAction::separator())
            .action(ToolbarAction::button("Another").tooltip("Test tooltip"));

        assert_eq!(toolbar.actions.len(), 3);
    }
}
