//! Color theme and styling constants

use ratatui::prelude::*;

/// Application color theme (Dracula-inspired)
pub struct Theme;

impl Theme {
    // Background colors
    pub const BG: Color = Color::Rgb(40, 42, 54);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(68, 71, 90);
    pub const BG_SELECTION: Color = Color::Rgb(68, 71, 90);
    
    // Foreground colors
    pub const FG: Color = Color::Rgb(248, 248, 242);
    pub const FG_DIM: Color = Color::Rgb(98, 114, 164);
    
    // Accent colors
    pub const ACCENT: Color = Color::Rgb(189, 147, 249);       // Purple
    pub const CYAN: Color = Color::Rgb(139, 233, 253);         // Cyan - directories
    pub const PINK: Color = Color::Rgb(255, 121, 198);         // Pink - files
    pub const GREEN: Color = Color::Rgb(80, 250, 123);         // Green - sizes
    pub const YELLOW: Color = Color::Rgb(241, 250, 140);       // Yellow
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);       // Orange - warning
    pub const RED: Color = Color::Rgb(255, 85, 85);            // Red - danger
    
    // Semantic colors
    pub const DIR: Color = Self::CYAN;
    pub const FILE: Color = Self::PINK;
    pub const SIZE: Color = Self::GREEN;
    pub const WARN: Color = Self::ORANGE;
    pub const DANGER: Color = Self::RED;
    pub const SUCCESS: Color = Self::GREEN;
    pub const INFO: Color = Self::CYAN;
    pub const MARKED: Color = Self::YELLOW;
    pub const ERROR: Color = Self::RED;
    
    // Bar gradient colors
    pub fn bar_color(percent: f64) -> Color {
        if percent > 80.0 {
            Self::DANGER
        } else if percent > 50.0 {
            Self::ORANGE
        } else if percent > 20.0 {
            Self::YELLOW
        } else {
            Self::GREEN
        }
    }
}

/// Common styles
pub mod styles {
    use super::*;
    
    pub fn normal() -> Style {
        Style::default().fg(Theme::FG).bg(Theme::BG)
    }
    
    pub fn highlight() -> Style {
        Style::default().fg(Theme::FG).bg(Theme::BG_SELECTION)
    }
    
    pub fn header() -> Style {
        Style::default()
            .fg(Theme::BG)
            .bg(Theme::CYAN)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn directory() -> Style {
        Style::default().fg(Theme::DIR)
    }
    
    pub fn file() -> Style {
        Style::default().fg(Theme::FILE)
    }
    
    pub fn size() -> Style {
        Style::default().fg(Theme::SIZE)
    }
    
    pub fn warning() -> Style {
        Style::default().fg(Theme::WARN)
    }
    
    pub fn danger() -> Style {
        Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD)
    }
    
    pub fn success() -> Style {
        Style::default().fg(Theme::SUCCESS)
    }
    
    pub fn dim() -> Style {
        Style::default().fg(Theme::FG_DIM)
    }
    
    pub fn marked() -> Style {
        Style::default().fg(Theme::MARKED).add_modifier(Modifier::BOLD)
    }
    
    pub fn border() -> Style {
        Style::default().fg(Theme::BG_HIGHLIGHT)
    }
    
    pub fn accent() -> Style {
        Style::default().fg(Theme::ACCENT)
    }
}