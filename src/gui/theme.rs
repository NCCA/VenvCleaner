//! Theme module for GUI styling and colors
//!
//! This module provides consistent theming and styling for the VenvCleaner GUI,
//! including color schemes, fonts, and layout constants.

use eframe::egui::{Color32, FontId, Rounding, Stroke, Style, Visuals};

/// Theme configuration for the GUI application
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary application colors
    pub primary: Color32,
    pub secondary: Color32,
    pub accent: Color32,

    /// Status colors
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,

    /// Background colors
    pub background: Color32,
    pub surface: Color32,
    pub panel: Color32,

    /// Text colors
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,

    /// Selection and highlight colors
    pub selection: Color32,
    pub highlight: Color32,
    pub hover: Color32,

    /// Border and separator colors
    pub border: Color32,
    pub separator: Color32,

    /// Age-based colors for .venv directories
    pub age_recent: Color32,     // Green for recently used
    pub age_moderate: Color32,   // Yellow for moderately used
    pub age_old: Color32,        // Red for old directories

    /// Size-based colors for file sizes
    pub size_small: Color32,     // Gray for small sizes
    pub size_medium: Color32,    // Orange for medium sizes
    pub size_large: Color32,     // Red for large sizes

    /// Font sizes
    pub font_small: f32,
    pub font_normal: f32,
    pub font_large: f32,
    pub font_heading: f32,

    /// Layout constants
    pub spacing: f32,
    pub padding: f32,
    pub button_height: f32,
    pub row_height: f32,

    /// Rounding and stroke settings
    pub rounding: Rounding,
    pub stroke_width: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Primary colors - blue theme
            primary: Color32::from_rgb(70, 130, 200),
            secondary: Color32::from_rgb(100, 160, 220),
            accent: Color32::from_rgb(255, 165, 0),

            // Status colors
            success: Color32::from_rgb(40, 180, 40),
            warning: Color32::from_rgb(255, 165, 0),
            error: Color32::from_rgb(220, 50, 50),
            info: Color32::from_rgb(70, 130, 200),

            // Background colors
            background: Color32::from_rgb(248, 249, 250),
            surface: Color32::from_rgb(255, 255, 255),
            panel: Color32::from_rgb(245, 246, 247),

            // Text colors
            text_primary: Color32::from_rgb(33, 37, 41),
            text_secondary: Color32::from_rgb(73, 80, 87),
            text_muted: Color32::from_rgb(134, 142, 150),

            // Selection and highlight colors
            selection: Color32::from_rgba_unmultiplied(70, 130, 200, 80),
            highlight: Color32::from_rgba_unmultiplied(255, 165, 0, 60),
            hover: Color32::from_rgba_unmultiplied(70, 130, 200, 40),

            // Border and separator colors
            border: Color32::from_rgb(206, 212, 218),
            separator: Color32::from_rgb(233, 236, 239),

            // Age-based colors (matching TUI colors)
            age_recent: Color32::from_rgb(40, 180, 40),      // Green
            age_moderate: Color32::from_rgb(255, 193, 7),    // Yellow
            age_old: Color32::from_rgb(220, 53, 69),         // Red

            // Size-based colors
            size_small: Color32::from_rgb(134, 142, 150),    // Gray
            size_medium: Color32::from_rgb(255, 165, 0),     // Orange
            size_large: Color32::from_rgb(220, 53, 69),      // Red

            // Font sizes
            font_small: 11.0,
            font_normal: 13.0,
            font_large: 16.0,
            font_heading: 20.0,

            // Layout constants
            spacing: 8.0,
            padding: 12.0,
            button_height: 32.0,
            row_height: 28.0,

            // Rounding and stroke
            rounding: Rounding::same(4.0),
            stroke_width: 1.0,
        }
    }
}

impl Theme {
    /// Create a dark theme variant
    pub fn dark() -> Self {
        Self {
            // Primary colors - slightly brighter for dark theme
            primary: Color32::from_rgb(100, 160, 240),
            secondary: Color32::from_rgb(130, 180, 250),
            accent: Color32::from_rgb(255, 193, 7),

            // Status colors - slightly adjusted for dark backgrounds
            success: Color32::from_rgb(72, 207, 72),
            warning: Color32::from_rgb(255, 193, 7),
            error: Color32::from_rgb(248, 81, 73),
            info: Color32::from_rgb(100, 160, 240),

            // Dark background colors
            background: Color32::from_rgb(32, 33, 36),
            surface: Color32::from_rgb(45, 46, 49),
            panel: Color32::from_rgb(40, 41, 44),

            // Light text colors for dark theme
            text_primary: Color32::from_rgb(255, 255, 255),
            text_secondary: Color32::from_rgb(189, 193, 198),
            text_muted: Color32::from_rgb(154, 160, 166),

            // Selection and highlight colors for dark theme
            selection: Color32::from_rgba_unmultiplied(100, 160, 240, 80),
            highlight: Color32::from_rgba_unmultiplied(255, 193, 7, 60),
            hover: Color32::from_rgba_unmultiplied(100, 160, 240, 40),

            // Dark theme borders
            border: Color32::from_rgb(95, 99, 104),
            separator: Color32::from_rgb(60, 64, 67),

            // Age colors remain the same as they work well on dark backgrounds
            age_recent: Color32::from_rgb(72, 207, 72),
            age_moderate: Color32::from_rgb(255, 193, 7),
            age_old: Color32::from_rgb(248, 81, 73),

            // Size colors adjusted for dark theme
            size_small: Color32::from_rgb(154, 160, 166),
            size_medium: Color32::from_rgb(255, 193, 7),
            size_large: Color32::from_rgb(248, 81, 73),

            ..Default::default()
        }
    }

    /// Apply this theme to the egui context
    pub fn apply_to_ctx(&self, ctx: &eframe::egui::Context) {
        let mut style = Style::default();

        // Set colors
        style.visuals.widgets.noninteractive.bg_fill = self.surface;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(self.stroke_width, self.text_primary);

        style.visuals.widgets.inactive.bg_fill = self.panel;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(self.stroke_width, self.text_secondary);

        style.visuals.widgets.hovered.bg_fill = self.hover;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(self.stroke_width, self.text_primary);

        style.visuals.widgets.active.bg_fill = self.selection;
        style.visuals.widgets.active.fg_stroke = Stroke::new(self.stroke_width, self.text_primary);

        style.visuals.selection.bg_fill = self.selection;
        style.visuals.selection.stroke = Stroke::new(self.stroke_width, self.primary);

        // Set panel background
        style.visuals.panel_fill = self.background;
        style.visuals.window_fill = self.surface;

        // Set rounding
        style.visuals.widgets.noninteractive.rounding = self.rounding;
        style.visuals.widgets.inactive.rounding = self.rounding;
        style.visuals.widgets.hovered.rounding = self.rounding;
        style.visuals.widgets.active.rounding = self.rounding;

        // Set spacing
        style.spacing.item_spacing = eframe::egui::Vec2::splat(self.spacing);
        style.spacing.button_padding = eframe::egui::Vec2::new(self.padding, self.padding * 0.5);

        ctx.set_style(style);
    }

    /// Get font ID for a specific size category
    pub fn font_id(&self, size: FontSize) -> FontId {
        let size = match size {
            FontSize::Small => self.font_small,
            FontSize::Normal => self.font_normal,
            FontSize::Large => self.font_large,
            FontSize::Heading => self.font_heading,
        };
        FontId::proportional(size)
    }

    /// Get color for .venv age in days
    pub fn get_age_color(&self, days: i64) -> Color32 {
        if days <= 30 {
            self.age_recent
        } else if days <= 90 {
            self.age_moderate
        } else {
            self.age_old
        }
    }

    /// Get color for file size in bytes
    pub fn get_size_color(&self, bytes: u64) -> Color32 {
        const MB_100: u64 = 100 * 1024 * 1024;
        const GB_1: u64 = 1024 * 1024 * 1024;

        if bytes >= GB_1 {
            self.size_large
        } else if bytes >= MB_100 {
            self.size_medium
        } else {
            self.size_small
        }
    }

    /// Get stroke for borders
    pub fn border_stroke(&self) -> Stroke {
        Stroke::new(self.stroke_width, self.border)
    }

    /// Get stroke for separators
    pub fn separator_stroke(&self) -> Stroke {
        Stroke::new(self.stroke_width * 0.5, self.separator)
    }
}

/// Font size categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Normal,
    Large,
    Heading,
}

/// Predefined color schemes
pub struct ColorSchemes;

impl ColorSchemes {
    /// Blue theme (default)
    pub fn blue() -> Theme {
        Theme::default()
    }

    /// Dark theme
    pub fn dark() -> Theme {
        Theme::dark()
    }

    /// Green theme
    pub fn green() -> Theme {
        Theme {
            primary: Color32::from_rgb(40, 167, 69),
            secondary: Color32::from_rgb(72, 180, 97),
            accent: Color32::from_rgb(255, 193, 7),
            selection: Color32::from_rgba_unmultiplied(40, 167, 69, 80),
            hover: Color32::from_rgba_unmultiplied(40, 167, 69, 40),
            ..Default::default()
        }
    }

    /// Purple theme
    pub fn purple() -> Theme {
        Theme {
            primary: Color32::from_rgb(108, 117, 225),
            secondary: Color32::from_rgb(130, 138, 235),
            accent: Color32::from_rgb(255, 193, 7),
            selection: Color32::from_rgba_unmultiplied(108, 117, 225, 80),
            hover: Color32::from_rgba_unmultiplied(108, 117, 225, 40),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.font_normal, 13.0);
        assert_eq!(theme.spacing, 8.0);
        assert_ne!(theme.primary, Color32::TRANSPARENT);
    }

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        // Dark theme should have dark background
        assert!(theme.background.r() < 100);
        assert!(theme.background.g() < 100);
        assert!(theme.background.b() < 100);
    }

    #[test]
    fn test_age_colors() {
        let theme = Theme::default();

        // Recent should be green-ish
        let recent_color = theme.get_age_color(15);
        assert_eq!(recent_color, theme.age_recent);

        // Moderate should be yellow-ish
        let moderate_color = theme.get_age_color(60);
        assert_eq!(moderate_color, theme.age_moderate);

        // Old should be red-ish
        let old_color = theme.get_age_color(120);
        assert_eq!(old_color, theme.age_old);
    }

    #[test]
    fn test_size_colors() {
        let theme = Theme::default();

        // Small size should be gray-ish
        let small_color = theme.get_size_color(1024); // 1KB
        assert_eq!(small_color, theme.size_small);

        // Medium size should be orange-ish
        let medium_color = theme.get_size_color(200 * 1024 * 1024); // 200MB
        assert_eq!(medium_color, theme.size_medium);

        // Large size should be red-ish
        let large_color = theme.get_size_color(2 * 1024 * 1024 * 1024); // 2GB
        assert_eq!(large_color, theme.size_large);
    }

    #[test]
    fn test_font_sizes() {
        let theme = Theme::default();

        let small_font = theme.font_id(FontSize::Small);
        let normal_font = theme.font_id(FontSize::Normal);
        let large_font = theme.font_id(FontSize::Large);
        let heading_font = theme.font_id(FontSize::Heading);

        // Fonts should be proportional and different sizes
        assert!(matches!(small_font, FontId { family: eframe::egui::FontFamily::Proportional, size } if size == theme.font_small));
        assert!(matches!(normal_font, FontId { family: eframe::egui::FontFamily::Proportional, size } if size == theme.font_normal));
        assert!(matches!(large_font, FontId { family: eframe::egui::FontFamily::Proportional, size } if size == theme.font_large));
        assert!(matches!(heading_font, FontId { family: eframe::egui::FontFamily::Proportional, size } if size == theme.font_heading));
    }

    #[test]
    fn test_color_schemes() {
        let blue = ColorSchemes::blue();
        let dark = ColorSchemes::dark();
        let green = ColorSchemes::green();
        let purple = ColorSchemes::purple();

        // Each scheme should have different primary colors
        assert_ne!(blue.primary, dark.primary);
        assert_ne!(blue.primary, green.primary);
        assert_ne!(blue.primary, purple.primary);
        assert_ne!(green.primary, purple.primary);
    }
}
