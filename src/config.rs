//! Application configuration with TOML support
//!
//! Configuration is loaded from ~/.config/vidu/config.toml if it exists.

use crate::app::{SortMode, SortOrder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Default config file path
fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vidu")
        .join("config.toml")
}

/// Application configuration (runtime)
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

    /// Junk detection settings
    pub junk: JunkConfig,

    /// Duplicate detection settings
    pub duplicates: DuplicatesConfig,

    /// Cleaning presets
    pub presets: PresetsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            force_fresh: false,
            show_hidden: true,  // Show all files by default for disk analysis
            sort_mode: SortMode::Size,
            sort_order: SortOrder::Descending,
            cache: CacheConfig::default(),
            ui: UiConfig::default(),
            junk: JunkConfig::default(),
            duplicates: DuplicatesConfig::default(),
            presets: PresetsConfig::default(),
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

    /// Load config from default location, merging with defaults
    pub fn load() -> Self {
        Self::load_from(&default_config_path())
    }

    /// Load config from a specific path, merging with defaults
    pub fn load_from(path: &Path) -> Self {
        let mut config = Self::default();

        if path.exists() {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(file_config) = toml::from_str::<FileConfig>(&contents) {
                    config.merge_file_config(file_config);
                }
            }
        }

        config
    }

    /// Merge file config into this config
    fn merge_file_config(&mut self, file: FileConfig) {
        if let Some(general) = file.general {
            if let Some(v) = general.show_hidden {
                self.show_hidden = v;
            }
            if let Some(v) = general.deletion_mode {
                self.junk.deletion_mode = v;
            }
            if let Some(v) = general.auto_detect_junk {
                self.junk.auto_detect = v;
            }
        }

        if let Some(cache) = file.cache {
            if let Some(v) = cache.enabled {
                self.cache.enabled = v;
            }
            if let Some(v) = cache.expiry_hours {
                self.cache.expiry_secs = v * 3600;
            }
            if let Some(v) = cache.compression {
                self.cache.use_compression = v;
            }
        }

        if let Some(ui) = file.ui {
            if let Some(v) = ui.mouse_enabled {
                self.ui.mouse_enabled = v;
            }
            if let Some(v) = ui.show_icons {
                self.ui.show_icons = v;
            }
            if let Some(v) = ui.show_bars {
                self.ui.show_bars = v;
            }
            if let Some(v) = ui.show_times {
                self.ui.show_times = v;
            }
            if let Some(v) = ui.theme {
                self.ui.theme = v;
            }
            if let Some(v) = ui.symbols {
                self.ui.symbols = v;
            }
        }

        if let Some(junk) = file.junk_detection {
            if let Some(v) = junk.junk_directories {
                self.junk.junk_directories = v;
            }
            if let Some(v) = junk.junk_files {
                self.junk.junk_files = v;
            }
            if let Some(v) = junk.custom_patterns {
                self.junk.custom_patterns = v;
            }
            if let Some(v) = junk.protected {
                self.junk.protected_paths = v;
            }
            if let Some(v) = junk.min_age_days {
                self.junk.min_age_days = v;
            }
        }

        if let Some(dups) = file.duplicates {
            if let Some(v) = dups.enabled {
                self.duplicates.enabled = v;
            }
            if let Some(v) = dups.min_size {
                self.duplicates.min_size = v;
            }
            if let Some(v) = dups.skip_hidden {
                self.duplicates.skip_hidden = v;
            }
            if let Some(v) = dups.exclude_patterns {
                self.duplicates.exclude_patterns = v;
            }
        }

        if let Some(presets) = file.presets {
            if let Some(custom) = presets.custom {
                self.presets.custom = custom;
            }
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

    /// Set path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    /// Save current config to default location
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&default_config_path())
    }

    /// Save current config to specific path
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        let file_config = FileConfig::from_config(self);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string =
            toml::to_string_pretty(&file_config).map_err(|e| std::io::Error::other(e))?;

        fs::write(path, toml_string)
    }
}

//=============================================================================
// File-based config structures (for TOML serialization)
//=============================================================================

/// Root config file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileConfig {
    general: Option<FileGeneralConfig>,
    cache: Option<FileCacheConfig>,
    ui: Option<FileUiConfig>,
    junk_detection: Option<FileJunkConfig>,
    duplicates: Option<FileDuplicatesConfig>,
    presets: Option<FilePresetsConfig>,
}

impl FileConfig {
    fn from_config(config: &Config) -> Self {
        Self {
            general: Some(FileGeneralConfig {
                show_hidden: Some(config.show_hidden),
                deletion_mode: Some(config.junk.deletion_mode),
                auto_detect_junk: Some(config.junk.auto_detect),
            }),
            cache: Some(FileCacheConfig {
                enabled: Some(config.cache.enabled),
                expiry_hours: Some(config.cache.expiry_secs / 3600),
                compression: Some(config.cache.use_compression),
            }),
            ui: Some(FileUiConfig {
                mouse_enabled: Some(config.ui.mouse_enabled),
                show_icons: Some(config.ui.show_icons),
                show_bars: Some(config.ui.show_bars),
                show_times: Some(config.ui.show_times),
                theme: Some(config.ui.theme.clone()),
                symbols: Some(config.ui.symbols),
            }),
            junk_detection: Some(FileJunkConfig {
                junk_directories: Some(config.junk.junk_directories.clone()),
                junk_files: Some(config.junk.junk_files.clone()),
                custom_patterns: Some(config.junk.custom_patterns.clone()),
                protected: Some(config.junk.protected_paths.clone()),
                min_age_days: Some(config.junk.min_age_days),
            }),
            duplicates: Some(FileDuplicatesConfig {
                enabled: Some(config.duplicates.enabled),
                min_size: Some(config.duplicates.min_size),
                skip_hidden: Some(config.duplicates.skip_hidden),
                exclude_patterns: Some(config.duplicates.exclude_patterns.clone()),
            }),
            presets: Some(FilePresetsConfig {
                custom: Some(config.presets.custom.clone()),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileGeneralConfig {
    show_hidden: Option<bool>,
    deletion_mode: Option<DeletionMode>,
    auto_detect_junk: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileCacheConfig {
    enabled: Option<bool>,
    expiry_hours: Option<u64>,
    compression: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileUiConfig {
    mouse_enabled: Option<bool>,
    show_icons: Option<bool>,
    show_bars: Option<bool>,
    show_times: Option<bool>,
    theme: Option<String>,
    symbols: Option<SymbolModeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileJunkConfig {
    junk_directories: Option<Vec<String>>,
    junk_files: Option<Vec<String>>,
    custom_patterns: Option<Vec<String>>,
    protected: Option<Vec<String>>,
    min_age_days: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileDuplicatesConfig {
    enabled: Option<bool>,
    min_size: Option<u64>,
    skip_hidden: Option<bool>,
    exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FilePresetsConfig {
    custom: Option<Vec<CleaningPreset>>,
}

//=============================================================================
// Runtime config structures
//=============================================================================

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

    /// Color theme name (e.g., "dracula", "nord", "gruvbox")
    pub theme: String,

    /// Symbol mode: "auto", "unicode", or "ascii"
    pub symbols: SymbolModeConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            mouse_enabled: true,
            show_icons: true,
            show_bars: true,
            show_times: true,
            theme: "dracula".into(),
            symbols: SymbolModeConfig::Auto,
        }
    }
}

/// Symbol mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SymbolModeConfig {
    /// Auto-detect based on terminal capabilities
    #[default]
    Auto,
    /// Force Unicode symbols (Nerd Fonts)
    Unicode,
    /// Force ASCII-only symbols
    Ascii,
}

/// Keybinding configuration
/// Allows customization of key bindings for various actions.
#[derive(Debug, Clone, Default)]
pub struct KeybindingsConfig {
    /// Key for quitting (default: q)
    pub quit: Vec<String>,
    /// Key for moving up (default: k, Up)
    pub move_up: Vec<String>,
    /// Key for moving down (default: j, Down)
    pub move_down: Vec<String>,
    /// Key for entering directory (default: Enter, l, Right)
    pub enter: Vec<String>,
    /// Key for going back (default: Backspace, h, Left)
    pub go_back: Vec<String>,
    /// Key for deleting (default: d, Delete)
    pub delete: Vec<String>,
    /// Key for toggling mark (default: Space)
    pub mark: Vec<String>,
    /// Key for showing help (default: ?)
    pub help: Vec<String>,
    /// Key for refreshing (default: r)
    pub refresh: Vec<String>,
    /// Key for cycling sort (default: s)
    pub sort: Vec<String>,
    /// Key for toggling hidden files (default: .)
    pub toggle_hidden: Vec<String>,
    /// Key for search (default: /)
    pub search: Vec<String>,
}

impl KeybindingsConfig {
    /// Get default keybindings
    pub fn defaults() -> Self {
        Self {
            quit: vec!["q".into()],
            move_up: vec!["k".into(), "Up".into()],
            move_down: vec!["j".into(), "Down".into()],
            enter: vec!["Enter".into(), "l".into(), "Right".into()],
            go_back: vec!["Backspace".into(), "h".into(), "Left".into()],
            delete: vec!["d".into(), "Delete".into()],
            mark: vec!["Space".into()],
            help: vec!["?".into()],
            refresh: vec!["r".into()],
            sort: vec!["s".into()],
            toggle_hidden: vec![".".into()],
            search: vec!["/".into()],
        }
    }
}

/// Deletion mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeletionMode {
    /// Move to system trash (safe, recoverable)
    #[default]
    Trash,
    /// Permanent deletion (not recoverable)
    Permanent,
}

impl DeletionMode {
    pub fn label(&self) -> &'static str {
        match self {
            DeletionMode::Trash => "Trash",
            DeletionMode::Permanent => "Permanent",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            DeletionMode::Trash => DeletionMode::Permanent,
            DeletionMode::Permanent => DeletionMode::Trash,
        }
    }
}

/// Junk detection configuration
#[derive(Debug, Clone)]
pub struct JunkConfig {
    /// Whether to auto-detect junk during scan
    pub auto_detect: bool,

    /// Default deletion mode
    pub deletion_mode: DeletionMode,

    /// Directory names to detect as junk (e.g., "node_modules", "target")
    pub junk_directories: Vec<String>,

    /// File patterns to detect as junk (e.g., "*.tmp", ".DS_Store")
    pub junk_files: Vec<String>,

    /// Custom glob patterns for junk detection
    pub custom_patterns: Vec<String>,

    /// Paths that should never be marked as junk
    pub protected_paths: Vec<String>,

    /// Minimum age in days for "old file" detection (0 = disabled)
    pub min_age_days: u64,
}

impl Default for JunkConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            deletion_mode: DeletionMode::Trash,
            junk_directories: vec![
                // Build artifacts
                "node_modules".into(),
                "target".into(),
                "__pycache__".into(),
                ".pytest_cache".into(),
                "dist".into(),
                "build".into(),
                ".next".into(),
                ".nuxt".into(),
                ".output".into(),
                ".parcel-cache".into(),
                ".turbo".into(),
                ".vite".into(),
                "vendor".into(), // Composer/Go vendor
                // Caches
                ".cache".into(),
                ".gradle".into(),
                ".m2".into(), // Maven
            ],
            junk_files: vec![
                // Temporary files
                "*.tmp".into(),
                "*.temp".into(),
                "*.swp".into(),
                "*.swo".into(),
                "*~".into(),
                "*.bak".into(),
                "*.backup".into(),
                "*.old".into(),
                // System junk
                ".DS_Store".into(),
                "Thumbs.db".into(),
                "desktop.ini".into(),
                ".Spotlight-V100".into(),
                ".Trashes".into(),
                "*.log".into(),
            ],
            custom_patterns: Vec::new(),
            protected_paths: vec![
                ".git".into(),
                ".ssh".into(),
                ".gnupg".into(),
                ".config".into(),
            ],
            min_age_days: 0, // Disabled by default
        }
    }
}

/// Duplicate detection configuration
#[derive(Debug, Clone)]
pub struct DuplicatesConfig {
    /// Whether duplicate detection is enabled
    pub enabled: bool,

    /// Minimum file size for duplicate detection (bytes)
    pub min_size: u64,

    /// Skip hidden files in duplicate detection
    pub skip_hidden: bool,

    /// Patterns to exclude from duplicate detection
    pub exclude_patterns: Vec<String>,
}

impl Default for DuplicatesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_size: 1024, // 1KB minimum
            skip_hidden: true,
            exclude_patterns: vec![
                ".git/**".into(),
                "node_modules/**".into(),
                "target/**".into(),
            ],
        }
    }
}

/// Cleaning presets configuration
#[derive(Debug, Clone)]
pub struct PresetsConfig {
    /// Custom user-defined presets
    pub custom: Vec<CleaningPreset>,
}

impl Default for PresetsConfig {
    fn default() -> Self {
        Self {
            custom: Vec::new(),
        }
    }
}

impl PresetsConfig {
    /// Get all available presets (built-in + custom)
    pub fn all_presets(&self) -> Vec<CleaningPreset> {
        let mut presets = CleaningPreset::builtin();
        presets.extend(self.custom.clone());
        presets
    }
}

/// A cleaning preset - predefined set of patterns to clean
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningPreset {
    /// Name of the preset
    pub name: String,

    /// Description of what this preset cleans
    #[serde(default)]
    pub description: String,

    /// Glob patterns to match
    pub patterns: Vec<String>,

    /// Minimum age in days (0 = any age)
    #[serde(default)]
    pub min_age_days: u64,

    /// Whether this is a built-in preset
    #[serde(skip)]
    pub builtin: bool,
}

impl CleaningPreset {
    /// Create a new custom preset
    pub fn new(name: impl Into<String>, patterns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            patterns,
            min_age_days: 0,
            builtin: false,
        }
    }

    /// Get built-in presets
    pub fn builtin() -> Vec<Self> {
        vec![
            Self {
                name: "Developer".into(),
                description: "Build artifacts and dependencies".into(),
                patterns: vec![
                    "**/node_modules".into(),
                    "**/target".into(),
                    "**/__pycache__".into(),
                    "**/dist".into(),
                    "**/build".into(),
                    "**/.next".into(),
                    "**/.gradle".into(),
                ],
                min_age_days: 0,
                builtin: true,
            },
            Self {
                name: "System Caches".into(),
                description: "System and application caches".into(),
                patterns: vec![
                    "**/.cache".into(),
                    "**/.npm/_cacache".into(),
                    "**/.cargo/registry/cache".into(),
                    "**/.local/share/Trash".into(),
                ],
                min_age_days: 0,
                builtin: true,
            },
            Self {
                name: "Temporary Files".into(),
                description: "Temp files, backups, and swap files".into(),
                patterns: vec![
                    "**/*.tmp".into(),
                    "**/*.temp".into(),
                    "**/*.swp".into(),
                    "**/*~".into(),
                    "**/*.bak".into(),
                    "**/*.log".into(),
                ],
                min_age_days: 0,
                builtin: true,
            },
            Self {
                name: "Old Files (30+ days)".into(),
                description: "Files not accessed in 30+ days".into(),
                patterns: vec!["**/*".into()],
                min_age_days: 30,
                builtin: true,
            },
            Self {
                name: "Old Files (90+ days)".into(),
                description: "Files not accessed in 90+ days".into(),
                patterns: vec!["**/*".into()],
                min_age_days: 90,
                builtin: true,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.show_hidden); // Show all files by default for disk analysis
        assert!(config.junk.auto_detect);
        assert_eq!(config.junk.deletion_mode, DeletionMode::Trash);
    }

    #[test]
    fn test_deletion_mode_toggle() {
        assert_eq!(DeletionMode::Trash.toggle(), DeletionMode::Permanent);
        assert_eq!(DeletionMode::Permanent.toggle(), DeletionMode::Trash);
    }

    #[test]
    fn test_builtin_presets() {
        let presets = CleaningPreset::builtin();
        assert!(!presets.is_empty());
        assert!(presets.iter().all(|p| p.builtin));
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[general]
show_hidden = true
deletion_mode = "permanent"
auto_detect_junk = false

[junk_detection]
junk_directories = ["node_modules", "target"]
protected = [".git"]
min_age_days = 30

[[presets.custom]]
name = "My Cleanup"
patterns = ["**/temp/**"]
min_age_days = 7
"#;

        let file_config: FileConfig = toml::from_str(toml_str).unwrap();
        let mut config = Config::default();
        config.merge_file_config(file_config);

        assert!(config.show_hidden);
        assert_eq!(config.junk.deletion_mode, DeletionMode::Permanent);
        assert!(!config.junk.auto_detect);
        assert_eq!(config.junk.min_age_days, 30);
        assert_eq!(config.presets.custom.len(), 1);
        assert_eq!(config.presets.custom[0].name, "My Cleanup");
    }
}
