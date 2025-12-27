//! Application configuration

use crate::app::{SortMode, SortOrder};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to analyze
    pub path: PathBuf,

    /// Force a fresh scan, ignoring cache
    pub force_fresh: bool,

    /// Show hidden files
    pub show_hidden: bool,

    /// Initial sort mode
    pub sort_mode: SortMode,

    /// Initial sort order
    pub sort_order: SortOrder,

    /// Cache settings
    pub cache: CacheConfig,

    /// UI settings
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            force_fresh: false,
            show_hidden: false,
            sort_mode: SortMode::Size,
            sort_order: SortOrder::Descending,
            cache: CacheConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Config {
    /// Create a new config with the given path
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    /// Set force fresh flag
    pub fn force_fresh(mut self, fresh: bool) -> Self {
        self.force_fresh = fresh;
        self
    }

    /// Set show hidden flag
    pub fn show_hidden(mut self, hidden: bool) -> Self {
        self.show_hidden = hidden;
        self
    }
}

/// Cache-related configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,

    /// Cache expiry in seconds (0 = never expire)
    pub expiry_secs: u64,

    /// Use compression for cache files
    pub use_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            expiry_secs: 24 * 60 * 60, // 24 hours
            use_compression: true,
        }
    }
}

/// UI-related configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    /// Enable mouse support
    pub mouse_enabled: bool,

    /// Show file icons
    pub show_icons: bool,

    /// Show percentage bars
    pub show_bars: bool,

    /// Show file modification times
    pub show_times: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            mouse_enabled: true,
            show_icons: true,
            show_bars: true,
            show_times: true,
        }
    }
}
