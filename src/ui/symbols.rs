//! Symbol sets for Unicode and ASCII rendering
//!
//! Provides consistent symbols across different terminal capabilities.

use super::terminal::SymbolMode;

/// Symbol set for UI elements
#[derive(Debug, Clone)]
pub struct Symbols {
    // File icons
    pub dir_open: &'static str,
    pub dir_closed: &'static str,
    pub file: &'static str,
    pub file_hidden: &'static str,
    pub symlink: &'static str,

    // Status icons
    pub check: &'static str,
    pub cross: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,

    // Navigation
    pub arrow_right: &'static str,
    pub arrow_left: &'static str,
    pub arrow_up: &'static str,
    pub arrow_down: &'static str,
    pub breadcrumb_sep: &'static str,

    // Selection
    pub selected: &'static str,
    pub unselected: &'static str,
    pub marked: &'static str,
    pub unmarked: &'static str,

    // Progress
    pub spinner: &'static [&'static str],
    pub bar_filled: &'static str,
    pub bar_empty: &'static str,
    pub bar_partial: &'static [&'static str],

    // Borders
    pub border_h: &'static str,
    pub border_v: &'static str,
    pub corner_tl: &'static str,
    pub corner_tr: &'static str,
    pub corner_bl: &'static str,
    pub corner_br: &'static str,

    // Disk status
    pub disk_ok: &'static str,
    pub disk_low: &'static str,
    pub disk_critical: &'static str,

    // Junk types
    pub junk_build: &'static str,
    pub junk_cache: &'static str,
    pub junk_temp: &'static str,
    pub junk_log: &'static str,
    pub junk_backup: &'static str,
    pub junk_system: &'static str,

    // File categories
    pub cat_document: &'static str,
    pub cat_image: &'static str,
    pub cat_video: &'static str,
    pub cat_audio: &'static str,
    pub cat_archive: &'static str,
    pub cat_code: &'static str,
    pub cat_data: &'static str,
    pub cat_executable: &'static str,

    // Misc
    pub ellipsis: &'static str,
    pub refresh: &'static str,
    pub trash: &'static str,
    pub delete_perm: &'static str,
    pub sort_asc: &'static str,
    pub sort_desc: &'static str,
}

/// Unicode symbols (Nerd Fonts recommended)
pub static UNICODE: Symbols = Symbols {
    // File icons (Nerd Font)
    dir_open: "󰝰",
    dir_closed: "󰉋",
    file: "󰈔",
    file_hidden: "󰘓",
    symlink: "󰌹",

    // Status icons
    check: "✓",
    cross: "✗",
    warning: "⚠",
    error: "✖",
    info: "ℹ",

    // Navigation
    arrow_right: "→",
    arrow_left: "←",
    arrow_up: "↑",
    arrow_down: "↓",
    breadcrumb_sep: " › ",

    // Selection
    selected: "▶",
    unselected: " ",
    marked: "◉",
    unmarked: "○",

    // Progress (braille spinner)
    spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    bar_filled: "█",
    bar_empty: "░",
    bar_partial: &["▏", "▎", "▍", "▌", "▋", "▊", "▉"],

    // Borders (rounded)
    border_h: "─",
    border_v: "│",
    corner_tl: "╭",
    corner_tr: "╮",
    corner_bl: "╰",
    corner_br: "╯",

    // Disk status
    disk_ok: "●",
    disk_low: "◐",
    disk_critical: "○",

    // Junk types
    junk_build: "󰏗",
    junk_cache: "󰃨",
    junk_temp: "󰃢",
    junk_log: "󰌱",
    junk_backup: "󰁯",
    junk_system: "󰃤",

    // File categories
    cat_document: "󰈙",
    cat_image: "󰋩",
    cat_video: "󰕧",
    cat_audio: "󰎆",
    cat_archive: "󰗄",
    cat_code: "󰅩",
    cat_data: "󰆼",
    cat_executable: "󰘔",

    // Misc
    ellipsis: "…",
    refresh: "⟳",
    trash: "🗑",
    delete_perm: "⌫",
    sort_asc: "▲",
    sort_desc: "▼",
};

/// ASCII-only symbols for compatibility
pub static ASCII: Symbols = Symbols {
    // File icons
    dir_open: "[D]",
    dir_closed: "[D]",
    file: "[F]",
    file_hidden: "[.]",
    symlink: "[L]",

    // Status icons
    check: "[+]",
    cross: "[x]",
    warning: "[!]",
    error: "[X]",
    info: "[i]",

    // Navigation
    arrow_right: "->",
    arrow_left: "<-",
    arrow_up: "^",
    arrow_down: "v",
    breadcrumb_sep: " / ",

    // Selection
    selected: ">",
    unselected: " ",
    marked: "[*]",
    unmarked: "[ ]",

    // Progress
    spinner: &["|", "/", "-", "\\"],
    bar_filled: "#",
    bar_empty: "-",
    bar_partial: &[".", ":", "|"],

    // Borders
    border_h: "-",
    border_v: "|",
    corner_tl: "+",
    corner_tr: "+",
    corner_bl: "+",
    corner_br: "+",

    // Disk status
    disk_ok: "[OK]",
    disk_low: "[LO]",
    disk_critical: "[!!]",

    // Junk types
    junk_build: "[B]",
    junk_cache: "[C]",
    junk_temp: "[T]",
    junk_log: "[L]",
    junk_backup: "[~]",
    junk_system: "[S]",

    // File categories
    cat_document: "[DOC]",
    cat_image: "[IMG]",
    cat_video: "[VID]",
    cat_audio: "[AUD]",
    cat_archive: "[ZIP]",
    cat_code: "[COD]",
    cat_data: "[DAT]",
    cat_executable: "[EXE]",

    // Misc
    ellipsis: "...",
    refresh: "[R]",
    trash: "[DEL]",
    delete_perm: "[RM]",
    sort_asc: "^",
    sort_desc: "v",
};

impl Symbols {
    /// Get the appropriate symbol set for the terminal mode
    pub fn for_mode(mode: SymbolMode) -> &'static Self {
        match mode {
            SymbolMode::Unicode => &UNICODE,
            SymbolMode::Ascii => &ASCII,
        }
    }

    /// Get spinner frame by index (wraps around)
    pub fn spinner_frame(&self, frame: usize) -> &'static str {
        self.spinner[frame % self.spinner.len()]
    }

    /// Get partial bar character for a percentage (0.0-1.0 within a cell)
    pub fn bar_partial_char(&self, fraction: f64) -> &'static str {
        if fraction <= 0.0 {
            self.bar_empty
        } else if fraction >= 1.0 {
            self.bar_filled
        } else {
            let idx = (fraction * self.bar_partial.len() as f64) as usize;
            self.bar_partial[idx.min(self.bar_partial.len() - 1)]
        }
    }
}

/// Global symbol set (set once at startup based on terminal capabilities)
static mut CURRENT_SYMBOLS: Option<&'static Symbols> = None;

/// Initialize the global symbol set
pub fn init(mode: SymbolMode) {
    unsafe {
        CURRENT_SYMBOLS = Some(Symbols::for_mode(mode));
    }
}

/// Get the current symbol set
pub fn symbols() -> &'static Symbols {
    unsafe { CURRENT_SYMBOLS.unwrap_or(&UNICODE) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_wrap() {
        assert_eq!(UNICODE.spinner_frame(0), "⠋");
        assert_eq!(UNICODE.spinner_frame(10), "⠋"); // wraps
        assert_eq!(ASCII.spinner_frame(0), "|");
        assert_eq!(ASCII.spinner_frame(4), "|"); // wraps
    }

    #[test]
    fn test_bar_partial() {
        assert_eq!(UNICODE.bar_partial_char(0.0), "░");
        assert_eq!(UNICODE.bar_partial_char(1.0), "█");
    }
}
