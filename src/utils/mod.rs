//! Utility functions and helpers

use crate::constants::files::{TEXT_EXTENSIONS, TEXT_PATTERNS};
use chrono::{DateTime, Local};
use humansize::{format_size, DECIMAL};
use ratatui::prelude::*;
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

/// Format bytes into human-readable size
#[inline]
pub fn format_bytes(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Format SystemTime into readable string
pub fn format_time(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => {
            let datetime: DateTime<Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "Unknown".to_string(),
    }
}

/// Get relative age string (e.g., "2 days ago", "3 months ago")
pub fn format_age(time: Option<SystemTime>) -> String {
    let Some(t) = time else {
        return "Unknown".to_string();
    };

    let now = SystemTime::now();
    let duration = match now.duration_since(t) {
        Ok(d) => d,
        Err(_) => return "Future".to_string(),
    };

    let secs = duration.as_secs();

    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;
    const MONTH: u64 = 30 * DAY;
    const YEAR: u64 = 365 * DAY;

    if secs < MINUTE {
        "Just now".to_string()
    } else if secs < HOUR {
        format!("{} min ago", secs / MINUTE)
    } else if secs < DAY {
        format!("{} hours ago", secs / HOUR)
    } else if secs < WEEK {
        format!("{} days ago", secs / DAY)
    } else if secs < MONTH {
        format!("{} weeks ago", secs / WEEK)
    } else if secs < YEAR {
        format!("{} months ago", secs / MONTH)
    } else {
        format!("{} years ago", secs / YEAR)
    }
}

/// Truncate string to fit width with ellipsis
pub fn truncate_str(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "...".chars().take(max_width).collect()
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if current_width + char_width + 3 > max_width {
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        result.push_str("...");
        result
    }
}

/// Create a smooth progress bar using Unicode block characters
pub fn create_smooth_bar(percent: f64, width: usize) -> String {
    const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

    let percent = percent.clamp(0.0, 100.0);
    let total_filled = (percent / 100.0) * width as f64;
    let full_blocks = total_filled.floor() as usize;
    let remainder = total_filled - full_blocks as f64;
    let partial_idx = (remainder * 8.0).round() as usize;

    let mut s = String::with_capacity(width);

    for _ in 0..full_blocks.min(width) {
        s.push(BLOCKS[8]);
    }

    if full_blocks < width {
        s.push(BLOCKS[partial_idx.min(8)]);
        for _ in 0..(width - full_blocks - 1) {
            s.push(' ');
        }
    }

    s
}

/// Get file type icon based on extension or name
pub fn get_file_icon(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return match name.to_lowercase().as_str() {
            ".git" => "",
            "node_modules" => "",
            "target" => "",
            ".cache" | "cache" => "",
            "downloads" => "",
            "documents" => "",
            "pictures" | "images" => "",
            "music" => "",
            "videos" => "",
            _ => "",
        };
    }

    // Get extension
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        // Programming
        "rs" => "",
        "py" => "",
        "js" | "mjs" | "cjs" => "",
        "ts" | "tsx" => "",
        "go" => "",
        "c" | "h" => "",
        "cpp" | "hpp" | "cc" => "",
        "java" => "",
        "rb" => "",
        "php" => "",
        "swift" => "",
        "kt" | "kts" => "",
        "scala" => "",
        "lua" => "",
        "r" => "",

        // Web
        "html" | "htm" => "",
        "css" | "scss" | "sass" | "less" => "",
        "vue" => "",
        "svelte" => "",

        // Config
        "json" => "",
        "yaml" | "yml" => "",
        "toml" => "",
        "xml" => "",
        "ini" | "conf" | "config" => "",

        // Documents
        "md" | "markdown" => "",
        "txt" => "",
        "pdf" => "",
        "doc" | "docx" => "",
        "xls" | "xlsx" => "",
        "ppt" | "pptx" => "",

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "svg" => "",

        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "",

        // Video
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" => "",

        // Archives
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" => "",

        // Executables
        "exe" | "msi" | "app" | "dmg" => "",
        "sh" | "bash" | "zsh" | "fish" => "",

        // Other
        "lock" => "",
        "log" => "",
        "db" | "sqlite" | "sqlite3" => "",
        "gitignore" | "gitattributes" => "",
        "dockerfile" => "",

        _ => "",
    }
}

/// Check if a file is likely text/readable (for preview)
pub fn is_text_file(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    // Check extension
    if let Some(ext) = name_lower.rsplit('.').next() {
        if TEXT_EXTENSIONS.contains(&ext) {
            return true;
        }
    }

    // Check filename patterns
    for pattern in TEXT_PATTERNS {
        if name_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Center a rectangle within another (percentage based)
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a fixed-size centered rectangle
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let width = width.min(r.width);
    let height = height.min(r.height);

    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.02 kB");
        assert_eq!(format_bytes(1_000_000), "1 MB");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 2), "hi");
        assert_eq!(truncate_str("a", 5), "a");
    }

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file("test.rs"));
        assert!(is_text_file("README.md"));
        assert!(is_text_file("Makefile"));
        assert!(!is_text_file("image.png"));
        assert!(!is_text_file("video.mp4"));
    }

    #[test]
    fn test_smooth_bar() {
        let bar = create_smooth_bar(50.0, 10);
        assert_eq!(bar.chars().count(), 10);

        let bar_0 = create_smooth_bar(0.0, 10);
        assert!(bar_0.trim().is_empty() || bar_0.chars().all(|c| c == ' ' || c == '▏'));

        let bar_100 = create_smooth_bar(100.0, 10);
        assert!(bar_100.chars().all(|c| c == '█'));
    }
}
