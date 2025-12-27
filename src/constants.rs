//! Application constants and configuration values

use std::time::Duration;

/// Application metadata
pub mod app {
    pub const NAME: &str = "vidu";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// UI-related constants
pub mod ui {
    /// Default visible rows in the table
    pub const DEFAULT_VISIBLE_ROWS: usize = 20;

    /// Event poll timeout
    pub const EVENT_POLL_TIMEOUT_MS: u64 = 100;

    /// Minimum terminal width
    pub const MIN_TERMINAL_WIDTH: u16 = 60;

    /// Minimum terminal height
    pub const MIN_TERMINAL_HEIGHT: u16 = 10;

    /// Name column percentage width
    pub const NAME_COLUMN_PERCENT: u16 = 35;

    /// Size bar width
    pub const SIZE_BAR_WIDTH: usize = 12;

    /// Max name display width percentage
    pub const MAX_NAME_WIDTH_PERCENT: f64 = 0.35;
}

/// Scanner-related constants
pub mod scanner {
    use super::Duration;

    /// Maximum depth for parallel scanning
    pub const PARALLEL_SCAN_DEPTH: usize = 4;

    /// Minimum entries for parallel processing
    pub const MIN_ENTRIES_FOR_PARALLEL: usize = 2;

    /// Progress update depth threshold
    pub const PROGRESS_UPDATE_DEPTH: usize = 3;

    /// Default scan timeout
    pub const SCAN_TIMEOUT: Duration = Duration::from_secs(3600);
}

/// Preview-related constants
pub mod preview {
    /// Maximum lines to read for preview
    pub const MAX_LINES: usize = 500;

    /// Maximum bytes to read for preview
    pub const MAX_BYTES: usize = 1_000_000;

    /// Page scroll amount
    pub const PAGE_SCROLL: usize = 20;
}

/// Cache-related constants
pub mod cache {
    use super::Duration;

    /// Cache expiry duration (24 hours)
    pub const EXPIRY_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

    /// Cache file extension
    pub const FILE_EXTENSION: &str = "bin";

    /// Application qualifier for cache directory
    pub const QUALIFIER: &str = "com";

    /// Application organization for cache directory
    pub const ORGANIZATION: &str = "vidu";
}

/// File type detection constants
pub mod files {
    /// Text file extensions for preview support
    pub const TEXT_EXTENSIONS: &[&str] = &[
        "txt", "md", "markdown", "rs", "py", "js", "ts", "jsx", "tsx",
        "html", "htm", "css", "scss", "sass", "less", "json", "yaml", "yml",
        "toml", "xml", "ini", "conf", "config", "sh", "bash", "zsh", "fish",
        "c", "h", "cpp", "hpp", "cc", "java", "kt", "go", "swift", "rb",
        "php", "lua", "r", "sql", "vim", "gitignore", "gitattributes",
        "dockerfile", "makefile", "cmake", "gradle", "properties", "env",
        "log", "csv", "tsv",
    ];

    /// Filename patterns that indicate text files
    pub const TEXT_PATTERNS: &[&str] = &[
        "makefile", "dockerfile", "cmakelists", "readme", "license", "changelog",
    ];
}
