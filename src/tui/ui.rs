//! UI rendering for the TUI mode
//!
//! This module handles all the UI rendering for the TUI interface, including
//! the main screen, dialogs, progress bars, and help screens.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph,
        Wrap, BorderType
    },
};

use super::{TuiApp, SortBy, AppState};

/// Colors used throughout the TUI
pub struct Colors;

impl Colors {
    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::Yellow;
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const MUTED: Color = Color::Gray;
    pub const SELECTED: Color = Color::Blue;
    pub const HIGHLIGHT: Color = Color::Magenta;
}

/// Draw the main browsing screen
pub fn draw_main_screen(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // Header
    draw_header(f, chunks[0], app);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // File list
            Constraint::Percentage(30), // Details panel
        ])
        .split(chunks[1]);

    // File list
    draw_venv_list(f, main_chunks[0], app);

    // Details panel
    draw_details_panel(f, main_chunks[1], app);

    // Footer
    draw_footer(f, chunks[2], app);
}

/// Draw the header section
fn draw_header(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let search_mode = if app.is_recursive() { " (Recursive)" } else { " (Current Dir)" };
    let title = format!("VenvCleaner - {}{}", app.current_directory().display(), search_mode);
    let sort_info = format!("Sort: {} {}",
        app.sort_by().display_name(),
        if app.sort_by() == SortBy::Size { "↓" } else { "↑" }
    );

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Colors::PRIMARY))
        .title(title);

    let title_paragraph = Paragraph::new("")
        .block(title_block);

    let sort_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Colors::SECONDARY))
        .title("Sort");

    let sort_paragraph = Paragraph::new(sort_info)
        .block(sort_block)
        .alignment(Alignment::Center);

    f.render_widget(title_paragraph, header_chunks[0]);
    f.render_widget(sort_paragraph, header_chunks[1]);
}

/// Draw the .venv directory list
fn draw_venv_list(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let venvs = app.venvs();
    let selected_index = app.selected_index();
    let selected_venvs = app.selected_venvs();

    // Get visible range without mutating app
    let list_height = area.height.saturating_sub(2); // Account for borders
    let visible_items = list_height as usize;

    let scroll_offset = app.scroll_offset();
    let start = scroll_offset;
    let end = (start + visible_items).min(venvs.len());
    let visible_venvs = &venvs[start..end];

    let items: Vec<ListItem> = visible_venvs
        .iter()
        .enumerate()
        .map(|(i, venv)| {
            let actual_index = start + i;
            let is_selected = actual_index == selected_index;
            let is_marked = selected_venvs.contains(&actual_index);

            let mut spans = vec![];

            // Selection indicator
            if is_marked {
                spans.push(Span::styled("✓ ", Style::default().fg(Colors::SUCCESS)));
            } else {
                spans.push(Span::raw("  "));
            }

            // Age indicator
            if venv.is_recently_used() {
                spans.push(Span::styled("🟢 ", Style::default()));
            } else if venv.is_old() {
                spans.push(Span::styled("🔴 ", Style::default()));
            } else {
                spans.push(Span::styled("🟡 ", Style::default()));
            }

            // Path
            let path_text = format_path_for_display(&venv.location(), 40);
            spans.push(Span::styled(
                format!("{:<40}", path_text),
                if is_selected {
                    Style::default().fg(Colors::HIGHLIGHT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }
            ));

            // Size
            let size_text = venv.size_formatted();
            let size_color = if venv.size_bytes() > 1024 * 1024 * 1024 {
                Colors::ERROR
            } else if venv.size_bytes() > 100 * 1024 * 1024 {
                Colors::WARNING
            } else {
                Colors::MUTED
            };

            spans.push(Span::styled(
                format!("{:>12}", size_text),
                Style::default().fg(size_color)
            ));

            // Last used
            let age_text = format!("{}d", venv.age_in_days());
            spans.push(Span::styled(
                format!("{:>6}", age_text),
                Style::default().fg(Colors::MUTED)
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list_title = format!(".venv Directories ({}/{})",
        venvs.len(),
        if selected_venvs.is_empty() {
            "none selected".to_string()
        } else {
            format!("{} selected", selected_venvs.len())
        }
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::PRIMARY))
                .title(list_title)
        )
        .highlight_style(
            Style::default()
                .bg(Colors::SELECTED)
                .add_modifier(Modifier::BOLD)
        );

    // Calculate the highlight index relative to the visible range
    let highlight_index = if selected_index >= start && selected_index < end {
        Some(selected_index - start)
    } else {
        None
    };

    let mut list_state = ListState::default();
    if let Some(index) = highlight_index {
        list_state.select(Some(index));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

/// Draw the details panel
fn draw_details_panel(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Selected item details
            Constraint::Percentage(40), // Summary statistics
        ])
        .split(area);

    // Selected item details
    draw_selected_details(f, chunks[0], app);

    // Summary statistics
    draw_summary_stats(f, chunks[1], app);
}

/// Draw details for the selected .venv directory
fn draw_selected_details(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let mut text = vec![];

    if let Some(venv) = app.selected_venv() {
        text.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(venv.path().display().to_string()),
        ]));

        text.push(Line::from(vec![
            Span::styled("Size: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(venv.size_formatted()),
        ]));

        text.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(venv.created_formatted()),
        ]));

        text.push(Line::from(vec![
            Span::styled("Last Used: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(venv.last_modified_formatted()),
        ]));

        text.push(Line::from(vec![
            Span::styled("Age: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(format!("{} days", venv.age_in_days())),
        ]));

        text.push(Line::from(""));

        // Status indicators
        if venv.is_recently_used() {
            text.push(Line::from(vec![
                Span::styled("🟢 ", Style::default()),
                Span::styled("Recently used", Style::default().fg(Colors::SUCCESS)),
            ]));
        } else if venv.is_old() {
            text.push(Line::from(vec![
                Span::styled("🔴 ", Style::default()),
                Span::styled("Old (>90 days)", Style::default().fg(Colors::ERROR)),
            ]));
        } else {
            text.push(Line::from(vec![
                Span::styled("🟡 ", Style::default()),
                Span::styled("Moderately used", Style::default().fg(Colors::WARNING)),
            ]));
        }

        if app.is_item_selected(app.selected_index()) {
            text.push(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Colors::SUCCESS)),
                Span::styled("Selected for deletion", Style::default().fg(Colors::SUCCESS)),
            ]));
        }
    } else {
        text.push(Line::from("No .venv directory selected"));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::SECONDARY))
                .title("Details")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw summary statistics
fn draw_summary_stats(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let stats = app.get_summary_stats();

    let text = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(format!("{} directories", stats.total_count)),
        ]),
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(format!("{} directories", stats.selected_count)),
        ]),
        Line::from(vec![
            Span::styled("Total Size: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(format_size(stats.total_size)),
        ]),
        Line::from(vec![
            Span::styled("Selected Size: ", Style::default().fg(Colors::SECONDARY)),
            Span::raw(format_size(stats.selected_size)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🟢 Recent: ", Style::default().fg(Colors::SUCCESS)),
            Span::raw(format!("{}", stats.recent_count)),
        ]),
        Line::from(vec![
            Span::styled("🔴 Old: ", Style::default().fg(Colors::ERROR)),
            Span::raw(format!("{}", stats.old_count)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::SECONDARY))
                .title("Summary")
        );

    f.render_widget(paragraph, area);
}

/// Draw the footer with status and shortcuts
fn draw_footer(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Status
            Constraint::Percentage(40), // Shortcuts
        ])
        .split(area);

    // Status
    let status_text = app.status();
    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::MUTED))
                .title("Status")
        );

    // Shortcuts - show context-sensitive help
    let shortcuts_text = match app.state() {
        AppState::Browsing => {
            if app.has_selected_items() {
                "h:Help r:Refresh Space:Toggle x:Delete s:Sort o:Open Ctrl+A:All Ctrl+D:None q:Quit"
            } else {
                "h:Help r:Refresh Space:Select s:Sort o:Open Ctrl+A:Select All q:Quit"
            }
        }
        _ => "h:Help r:Refresh Space:Select x:Delete s:Sort o:Open q:Quit"
    };
    let shortcuts_paragraph = Paragraph::new(shortcuts_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::MUTED))
                .title("Shortcuts")
        );

    f.render_widget(status_paragraph, chunks[0]);
    f.render_widget(shortcuts_paragraph, chunks[1]);
}

/// Draw the loading screen
pub fn draw_loading_screen(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(60),
        ])
        .split(area);

    let loading_text = format!("Loading{}", ".".repeat(app.loading_dots()));
    let loading_paragraph = Paragraph::new(loading_text)
        .style(Style::default().fg(Colors::PRIMARY))
        .alignment(Alignment::Center);

    let status_text = app.status();
    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().fg(Colors::MUTED))
        .alignment(Alignment::Center);

    f.render_widget(loading_paragraph, chunks[1]);
    f.render_widget(status_paragraph, chunks[2]);
}

/// Draw the confirmation dialog
pub fn draw_confirmation_dialog(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let selected_count = app.selected_venvs().len();
    let selected_venvs = app.get_selected_venvs();
    let total_size: u64 = selected_venvs.iter().map(|v| v.size_bytes()).sum();

    // Calculate dialog size
    let dialog_width = 60;
    let dialog_height = 12;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠️  Confirm Deletion", Style::default().fg(Colors::WARNING).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("You are about to permanently delete "),
            Span::styled(format!("{}", selected_count), Style::default().fg(Colors::ERROR).add_modifier(Modifier::BOLD)),
            Span::raw(" .venv directories."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Total size to be freed: "),
            Span::styled(format_size(total_size), Style::default().fg(Colors::WARNING).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠️  This action cannot be undone!", Style::default().fg(Colors::ERROR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("The list will automatically refresh after deletion."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 'y' to confirm deletion or 'n'/Esc to cancel", Style::default().fg(Colors::MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::ERROR))
                .border_type(BorderType::Double)
                .title("Confirm Deletion")
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, dialog_area);
}

/// Draw the deletion progress dialog
pub fn draw_deletion_progress(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let progress = app.deletion_progress();

    // Calculate dialog size
    let dialog_width = 50;
    let dialog_height = 8;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    f.render_widget(Clear, dialog_area);

    let progress_ratio = if progress.total > 0 {
        progress.completed as f64 / progress.total as f64
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::PRIMARY))
                .title("Deleting...")
        )
        .gauge_style(Style::default().fg(Colors::SUCCESS))
        .ratio(progress_ratio)
        .label(format!("{}/{}", progress.completed, progress.total));

    f.render_widget(gauge, dialog_area);
}

/// Draw the error screen
pub fn draw_error_screen(f: &mut ratatui::Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Percentage(60),
        ])
        .split(area);

    let error_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("❌ Error", Style::default().fg(Colors::ERROR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(app.error_message()),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Enter to continue or 'q' to quit", Style::default().fg(Colors::MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(error_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::ERROR))
                .title("Error")
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, chunks[1]);
}

/// Draw the help screen
pub fn draw_help_screen(f: &mut ratatui::Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("VenvCleaner TUI Help", Style::default().fg(Colors::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Colors::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ↑/↓       - Move selection up/down"),
        Line::from("  Home/End  - Go to first/last item"),
        Line::from("  PgUp/PgDn - Page up/down"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Selection:", Style::default().fg(Colors::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space/Enter - Toggle selection"),
        Line::from("  Ctrl+A      - Select all"),
        Line::from("  Ctrl+D      - Deselect all"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(Colors::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  x/Del    - Delete selected items"),
        Line::from("  s        - Cycle sort order"),
        Line::from("  o        - Open folder in file manager"),
        Line::from("  r        - Refresh list"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Other:", Style::default().fg(Colors::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  h/F1     - Show this help"),
        Line::from("  q/Esc    - Quit application"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status Icons:", Style::default().fg(Colors::SECONDARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  🟢 - Recently used (<30 days)"),
        Line::from("  🟡 - Moderately used (30-90 days)"),
        Line::from("  🔴 - Old (>90 days)"),
        Line::from("  ✓  - Selected for deletion"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to return...", Style::default().fg(Colors::MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Colors::PRIMARY))
                .title("Help")
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Format a file path for display, truncating if necessary
fn format_path_for_display(path: &str, max_length: usize) -> String {
    if path.len() <= max_length {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_length - 3)..])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_path_for_display() {
        assert_eq!(format_path_for_display("short", 10), "short");
        assert_eq!(format_path_for_display("very/long/path/here", 10), "...th/here");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 bytes");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
