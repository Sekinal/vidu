//! Color theme and styling system
//!
//! Supports multiple built-in themes and custom theme loading from config.

use ratatui::prelude::*;
use std::sync::OnceLock;

/// Global theme instance
static CURRENT_THEME: OnceLock<ColorTheme> = OnceLock::new();

/// Color palette for a theme
#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub name: String,

    // Background colors
    pub bg: Color,
    pub bg_highlight: Color,
    pub bg_selection: Color,
    pub bg_alt: Color, // Alternating row background

    // Foreground colors
    pub fg: Color,
    pub fg_dim: Color,

    // Semantic colors
    pub accent: Color,
    pub directory: Color,
    pub file: Color,
    pub size: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub info: Color,
    pub marked: Color,

    // Bar gradient
    pub bar_low: Color,    // 0-20%
    pub bar_medium: Color, // 20-50%
    pub bar_high: Color,   // 50-80%
    pub bar_critical: Color, // 80-100%
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::dracula()
    }
}

impl ColorTheme {
    /// Dracula theme (default)
    pub fn dracula() -> Self {
        Self {
            name: "dracula".into(),
            bg: Color::Rgb(40, 42, 54),
            bg_highlight: Color::Rgb(68, 71, 90),
            bg_selection: Color::Rgb(68, 71, 90),
            bg_alt: Color::Rgb(44, 46, 60),
            fg: Color::Rgb(248, 248, 242),
            fg_dim: Color::Rgb(98, 114, 164),
            accent: Color::Rgb(189, 147, 249),
            directory: Color::Rgb(139, 233, 253),
            file: Color::Rgb(255, 121, 198),
            size: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 184, 108),
            danger: Color::Rgb(255, 85, 85),
            success: Color::Rgb(80, 250, 123),
            info: Color::Rgb(139, 233, 253),
            marked: Color::Rgb(241, 250, 140),
            bar_low: Color::Rgb(80, 250, 123),
            bar_medium: Color::Rgb(241, 250, 140),
            bar_high: Color::Rgb(255, 184, 108),
            bar_critical: Color::Rgb(255, 85, 85),
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "nord".into(),
            bg: Color::Rgb(46, 52, 64),
            bg_highlight: Color::Rgb(59, 66, 82),
            bg_selection: Color::Rgb(67, 76, 94),
            bg_alt: Color::Rgb(52, 58, 70),
            fg: Color::Rgb(236, 239, 244),
            fg_dim: Color::Rgb(76, 86, 106),
            accent: Color::Rgb(136, 192, 208),
            directory: Color::Rgb(129, 161, 193),
            file: Color::Rgb(163, 190, 140),
            size: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            danger: Color::Rgb(191, 97, 106),
            success: Color::Rgb(163, 190, 140),
            info: Color::Rgb(129, 161, 193),
            marked: Color::Rgb(235, 203, 139),
            bar_low: Color::Rgb(163, 190, 140),
            bar_medium: Color::Rgb(235, 203, 139),
            bar_high: Color::Rgb(208, 135, 112),
            bar_critical: Color::Rgb(191, 97, 106),
        }
    }

    /// Gruvbox dark theme
    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".into(),
            bg: Color::Rgb(40, 40, 40),
            bg_highlight: Color::Rgb(60, 56, 54),
            bg_selection: Color::Rgb(80, 73, 69),
            bg_alt: Color::Rgb(50, 48, 47),
            fg: Color::Rgb(235, 219, 178),
            fg_dim: Color::Rgb(146, 131, 116),
            accent: Color::Rgb(211, 134, 155),
            directory: Color::Rgb(131, 165, 152),
            file: Color::Rgb(184, 187, 38),
            size: Color::Rgb(142, 192, 124),
            warning: Color::Rgb(250, 189, 47),
            danger: Color::Rgb(251, 73, 52),
            success: Color::Rgb(142, 192, 124),
            info: Color::Rgb(131, 165, 152),
            marked: Color::Rgb(250, 189, 47),
            bar_low: Color::Rgb(142, 192, 124),
            bar_medium: Color::Rgb(250, 189, 47),
            bar_high: Color::Rgb(254, 128, 25),
            bar_critical: Color::Rgb(251, 73, 52),
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".into(),
            bg: Color::Rgb(30, 30, 46),
            bg_highlight: Color::Rgb(49, 50, 68),
            bg_selection: Color::Rgb(69, 71, 90),
            bg_alt: Color::Rgb(36, 36, 54),
            fg: Color::Rgb(205, 214, 244),
            fg_dim: Color::Rgb(108, 112, 134),
            accent: Color::Rgb(203, 166, 247),
            directory: Color::Rgb(137, 180, 250),
            file: Color::Rgb(245, 194, 231),
            size: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            danger: Color::Rgb(243, 139, 168),
            success: Color::Rgb(166, 227, 161),
            info: Color::Rgb(137, 180, 250),
            marked: Color::Rgb(249, 226, 175),
            bar_low: Color::Rgb(166, 227, 161),
            bar_medium: Color::Rgb(249, 226, 175),
            bar_high: Color::Rgb(250, 179, 135),
            bar_critical: Color::Rgb(243, 139, 168),
        }
    }

    /// Solarized dark theme
    pub fn solarized() -> Self {
        Self {
            name: "solarized".into(),
            bg: Color::Rgb(0, 43, 54),
            bg_highlight: Color::Rgb(7, 54, 66),
            bg_selection: Color::Rgb(7, 54, 66),
            bg_alt: Color::Rgb(0, 48, 60),
            fg: Color::Rgb(131, 148, 150),
            fg_dim: Color::Rgb(88, 110, 117),
            accent: Color::Rgb(108, 113, 196),
            directory: Color::Rgb(38, 139, 210),
            file: Color::Rgb(211, 54, 130),
            size: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            danger: Color::Rgb(220, 50, 47),
            success: Color::Rgb(133, 153, 0),
            info: Color::Rgb(42, 161, 152),
            marked: Color::Rgb(181, 137, 0),
            bar_low: Color::Rgb(133, 153, 0),
            bar_medium: Color::Rgb(181, 137, 0),
            bar_high: Color::Rgb(203, 75, 22),
            bar_critical: Color::Rgb(220, 50, 47),
        }
    }

    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night".into(),
            bg: Color::Rgb(26, 27, 38),
            bg_highlight: Color::Rgb(41, 46, 66),
            bg_selection: Color::Rgb(51, 59, 81),
            bg_alt: Color::Rgb(30, 31, 44),
            fg: Color::Rgb(169, 177, 214),
            fg_dim: Color::Rgb(86, 95, 137),
            accent: Color::Rgb(187, 154, 247),
            directory: Color::Rgb(125, 207, 255),
            file: Color::Rgb(247, 118, 142),
            size: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            danger: Color::Rgb(247, 118, 142),
            success: Color::Rgb(158, 206, 106),
            info: Color::Rgb(125, 207, 255),
            marked: Color::Rgb(224, 175, 104),
            bar_low: Color::Rgb(158, 206, 106),
            bar_medium: Color::Rgb(224, 175, 104),
            bar_high: Color::Rgb(255, 158, 100),
            bar_critical: Color::Rgb(247, 118, 142),
        }
    }

    /// High contrast monochrome theme
    pub fn mono() -> Self {
        Self {
            name: "mono".into(),
            bg: Color::Black,
            bg_highlight: Color::Rgb(30, 30, 30),
            bg_selection: Color::Rgb(50, 50, 50),
            bg_alt: Color::Rgb(15, 15, 15),
            fg: Color::White,
            fg_dim: Color::Gray,
            accent: Color::White,
            directory: Color::White,
            file: Color::Gray,
            size: Color::White,
            warning: Color::LightYellow,
            danger: Color::LightRed,
            success: Color::LightGreen,
            info: Color::LightCyan,
            marked: Color::LightYellow,
            bar_low: Color::Green,
            bar_medium: Color::Yellow,
            bar_high: Color::LightRed,
            bar_critical: Color::Red,
        }
    }

    /// Get a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "gruvbox" => Some(Self::gruvbox()),
            "catppuccin" => Some(Self::catppuccin()),
            "solarized" => Some(Self::solarized()),
            "tokyo-night" | "tokyo_night" | "tokyonight" => Some(Self::tokyo_night()),
            "mono" | "monochrome" => Some(Self::mono()),
            _ => None,
        }
    }

    /// List all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "dracula",
            "nord",
            "gruvbox",
            "catppuccin",
            "solarized",
            "tokyo-night",
            "mono",
        ]
    }

    /// Get bar color for a percentage
    pub fn bar_color(&self, percent: f64) -> Color {
        if percent > 80.0 {
            self.bar_critical
        } else if percent > 50.0 {
            self.bar_high
        } else if percent > 20.0 {
            self.bar_medium
        } else {
            self.bar_low
        }
    }
}

/// Initialize the global theme
pub fn init(theme: ColorTheme) {
    let _ = CURRENT_THEME.set(theme);
}

/// Initialize with a theme name
pub fn init_by_name(name: &str) {
    let theme = ColorTheme::by_name(name).unwrap_or_default();
    init(theme);
}

/// Get the current theme
pub fn theme() -> &'static ColorTheme {
    CURRENT_THEME.get().unwrap_or_else(|| {
        // Return a static reference to dracula as fallback
        static FALLBACK: OnceLock<ColorTheme> = OnceLock::new();
        FALLBACK.get_or_init(ColorTheme::dracula)
    })
}

// ============================================================
// Legacy compatibility layer (maps to current theme)
// ============================================================

/// Legacy Theme struct for backwards compatibility
pub struct Theme;

impl Theme {
    pub const BG: Color = Color::Rgb(40, 42, 54);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(68, 71, 90);
    pub const BG_SELECTION: Color = Color::Rgb(68, 71, 90);
    pub const FG: Color = Color::Rgb(248, 248, 242);
    pub const FG_DIM: Color = Color::Rgb(98, 114, 164);
    pub const ACCENT: Color = Color::Rgb(189, 147, 249);
    pub const CYAN: Color = Color::Rgb(139, 233, 253);
    pub const PINK: Color = Color::Rgb(255, 121, 198);
    pub const GREEN: Color = Color::Rgb(80, 250, 123);
    pub const YELLOW: Color = Color::Rgb(241, 250, 140);
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);
    pub const RED: Color = Color::Rgb(255, 85, 85);

    pub const DIR: Color = Self::CYAN;
    pub const FILE: Color = Self::PINK;
    pub const SIZE: Color = Self::GREEN;
    pub const WARN: Color = Self::ORANGE;
    pub const DANGER: Color = Self::RED;
    pub const SUCCESS: Color = Self::GREEN;
    pub const INFO: Color = Self::CYAN;
    pub const MARKED: Color = Self::YELLOW;
    pub const ERROR: Color = Self::RED;

    pub fn bar_color(percent: f64) -> Color {
        theme().bar_color(percent)
    }
}

/// Common styles (now theme-aware)
pub mod styles {
    use super::*;

    pub fn normal() -> Style {
        let t = theme();
        Style::default().fg(t.fg).bg(t.bg)
    }

    pub fn highlight() -> Style {
        let t = theme();
        Style::default().fg(t.fg).bg(t.bg_selection)
    }

    pub fn header() -> Style {
        let t = theme();
        Style::default()
            .fg(t.bg)
            .bg(t.info)
            .add_modifier(Modifier::BOLD)
    }

    pub fn directory() -> Style {
        Style::default().fg(theme().directory)
    }

    pub fn file() -> Style {
        Style::default().fg(theme().file)
    }

    pub fn size() -> Style {
        Style::default().fg(theme().size)
    }

    pub fn warning() -> Style {
        Style::default().fg(theme().warning)
    }

    pub fn danger() -> Style {
        Style::default()
            .fg(theme().danger)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success() -> Style {
        Style::default().fg(theme().success)
    }

    pub fn dim() -> Style {
        Style::default().fg(theme().fg_dim)
    }

    pub fn marked() -> Style {
        Style::default()
            .fg(theme().marked)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(theme().bg_highlight)
    }

    pub fn accent() -> Style {
        Style::default().fg(theme().accent)
    }

    /// Style for alternating rows
    pub fn alt_row() -> Style {
        Style::default().bg(theme().bg_alt)
    }

    /// Style for marked row background
    pub fn marked_bg() -> Style {
        let t = theme();
        Style::default().bg(t.bg_selection)
    }

    /// Style for error row background
    pub fn error_bg() -> Style {
        let t = theme();
        Style::default().bg(Color::Rgb(60, 30, 30)) // Subtle red tint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_by_name() {
        assert!(ColorTheme::by_name("dracula").is_some());
        assert!(ColorTheme::by_name("nord").is_some());
        assert!(ColorTheme::by_name("invalid").is_none());
    }

    #[test]
    fn test_bar_colors() {
        let theme = ColorTheme::dracula();
        assert_eq!(theme.bar_color(10.0), theme.bar_low);
        assert_eq!(theme.bar_color(30.0), theme.bar_medium);
        assert_eq!(theme.bar_color(60.0), theme.bar_high);
        assert_eq!(theme.bar_color(90.0), theme.bar_critical);
    }

    #[test]
    fn test_available_themes() {
        let themes = ColorTheme::available_themes();
        assert!(themes.contains(&"dracula"));
        assert!(themes.len() >= 6);
    }
}
