//! Terminal capability detection
//!
//! Detects terminal features like Unicode support, color depth,
//! and adapts the UI accordingly.

use std::env;

/// Terminal capabilities detected at runtime
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    /// Whether the terminal supports Unicode symbols
    pub unicode: bool,
    /// Whether the terminal supports 24-bit true color
    pub true_color: bool,
    /// Whether the terminal supports 256 colors
    pub color_256: bool,
    /// Terminal width in columns
    pub width: u16,
    /// Terminal height in rows
    pub height: u16,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl TerminalCapabilities {
    /// Detect terminal capabilities from environment
    pub fn detect() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));

        Self {
            unicode: Self::detect_unicode(),
            true_color: Self::detect_true_color(),
            color_256: Self::detect_256_color(),
            width,
            height,
        }
    }

    /// Check if terminal supports Unicode
    fn detect_unicode() -> bool {
        // Check LANG and LC_* environment variables
        let lang = env::var("LANG").unwrap_or_default();
        let lc_all = env::var("LC_ALL").unwrap_or_default();
        let lc_ctype = env::var("LC_CTYPE").unwrap_or_default();

        // Look for UTF-8 indicator
        let has_utf8 = lang.to_lowercase().contains("utf")
            || lc_all.to_lowercase().contains("utf")
            || lc_ctype.to_lowercase().contains("utf");

        // Also check terminal type
        let term = env::var("TERM").unwrap_or_default();
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();

        // Modern terminals generally support Unicode
        let modern_term = term.contains("xterm")
            || term.contains("screen")
            || term.contains("tmux")
            || term.contains("alacritty")
            || term.contains("kitty")
            || term.contains("wezterm")
            || term_program.contains("iTerm")
            || term_program.contains("Apple_Terminal")
            || term_program.contains("vscode");

        has_utf8 || modern_term
    }

    /// Check if terminal supports 24-bit true color
    fn detect_true_color() -> bool {
        // COLORTERM is the most reliable indicator
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }

        // Check TERM for true color terminals
        let term = env::var("TERM").unwrap_or_default();
        if term.contains("truecolor") || term.contains("24bit") {
            return true;
        }

        // Known true color terminals
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let known_truecolor = [
            "alacritty",
            "kitty",
            "wezterm",
            "iTerm.app",
            "Apple_Terminal",
            "vscode",
            "Hyper",
        ];

        known_truecolor.iter().any(|t| term_program.contains(t) || term.contains(t))
    }

    /// Check if terminal supports 256 colors
    fn detect_256_color() -> bool {
        let term = env::var("TERM").unwrap_or_default();

        // 256color in TERM
        if term.contains("256color") || term.contains("256") {
            return true;
        }

        // Most modern terminals support 256 colors
        let modern_terms = ["xterm", "screen", "tmux", "rxvt", "linux"];
        modern_terms.iter().any(|t| term.contains(t))
    }

    /// Check if terminal size is sufficient for full UI
    pub fn is_size_adequate(&self) -> bool {
        self.width >= 60 && self.height >= 15
    }

    /// Check if terminal is very small
    pub fn is_compact_mode(&self) -> bool {
        self.width < 80 || self.height < 20
    }

    /// Check if terminal is tiny (minimal mode)
    pub fn is_minimal_mode(&self) -> bool {
        self.width < 60 || self.height < 15
    }

    /// Get recommended symbol set based on capabilities
    pub fn symbol_mode(&self) -> SymbolMode {
        if self.unicode {
            SymbolMode::Unicode
        } else {
            SymbolMode::Ascii
        }
    }

    /// Get recommended color mode based on capabilities
    pub fn color_mode(&self) -> ColorMode {
        if self.true_color {
            ColorMode::TrueColor
        } else if self.color_256 {
            ColorMode::Color256
        } else {
            ColorMode::Basic
        }
    }

    /// Refresh terminal size
    pub fn refresh_size(&mut self) {
        if let Ok((w, h)) = crossterm::terminal::size() {
            self.width = w;
            self.height = h;
        }
    }
}

/// Symbol rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolMode {
    /// Use Unicode symbols (Nerd Fonts, emoji, etc.)
    Unicode,
    /// Use ASCII-only symbols for compatibility
    Ascii,
}

/// Color rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// 24-bit true color (16 million colors)
    TrueColor,
    /// 256 color palette
    Color256,
    /// Basic 16 colors
    Basic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_capabilities() {
        let caps = TerminalCapabilities::detect();
        // Basic sanity checks
        assert!(caps.width > 0);
        assert!(caps.height > 0);
    }

    #[test]
    fn test_size_modes() {
        let mut caps = TerminalCapabilities::detect();

        caps.width = 100;
        caps.height = 30;
        assert!(caps.is_size_adequate());
        assert!(!caps.is_compact_mode());
        assert!(!caps.is_minimal_mode());

        caps.width = 70;
        caps.height = 18;
        assert!(caps.is_size_adequate());
        assert!(caps.is_compact_mode());
        assert!(!caps.is_minimal_mode());

        caps.width = 50;
        caps.height = 12;
        assert!(!caps.is_size_adequate());
        assert!(caps.is_compact_mode());
        assert!(caps.is_minimal_mode());
    }
}
