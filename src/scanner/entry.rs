//! Entry struct representing a file or directory

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};

/// File category based on content type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FileCategory {
    #[default]
    Unknown,
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Data,
    Executable,
    Font,
    Config,
}

impl FileCategory {
    /// Detect category from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Documents
            "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "tex" | "epub" => {
                FileCategory::Document
            }
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "raw"
            | "psd" | "ai" => FileCategory::Image,
            // Video
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" => {
                FileCategory::Video
            }
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => FileCategory::Audio,
            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" | "lz4" => {
                FileCategory::Archive
            }
            // Code
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp" | "go"
            | "rb" | "php" | "swift" | "kt" | "scala" | "cs" | "fs" | "hs" | "ml" | "clj"
            | "ex" | "exs" | "erl" | "lua" | "r" | "jl" | "nim" | "zig" | "v" | "d" | "sql"
            | "sh" | "bash" | "zsh" | "fish" | "ps1" | "html" | "css" | "scss" | "sass"
            | "less" | "vue" | "svelte" => FileCategory::Code,
            // Data
            "json" | "xml" | "yaml" | "yml" | "csv" | "tsv" | "parquet" | "sqlite" | "db" => {
                FileCategory::Data
            }
            // Executables
            "exe" | "dll" | "so" | "dylib" | "app" | "apk" | "deb" | "rpm" | "msi" | "dmg" => {
                FileCategory::Executable
            }
            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "eot" => FileCategory::Font,
            // Config
            "toml" | "ini" | "conf" | "cfg" | "env" | "properties" => FileCategory::Config,
            _ => FileCategory::Unknown,
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            FileCategory::Unknown => "Other",
            FileCategory::Document => "Documents",
            FileCategory::Image => "Images",
            FileCategory::Video => "Videos",
            FileCategory::Audio => "Audio",
            FileCategory::Archive => "Archives",
            FileCategory::Code => "Code",
            FileCategory::Data => "Data",
            FileCategory::Executable => "Executables",
            FileCategory::Font => "Fonts",
            FileCategory::Config => "Config",
        }
    }

    /// Get icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            FileCategory::Unknown => "󰈔",
            FileCategory::Document => "󰈙",
            FileCategory::Image => "󰋩",
            FileCategory::Video => "󰕧",
            FileCategory::Audio => "󰎆",
            FileCategory::Archive => "󰗄",
            FileCategory::Code => "󰅩",
            FileCategory::Data => "󰆼",
            FileCategory::Executable => "󰘔",
            FileCategory::Font => "󰛖",
            FileCategory::Config => "󰒓",
        }
    }
}

/// Type of junk/cleanable content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JunkType {
    /// Build artifacts (node_modules, target/, __pycache__, etc.)
    BuildArtifact,
    /// Cache directories (.cache/, browser caches, etc.)
    Cache,
    /// Temporary files (*.tmp, *.swp, etc.)
    Temporary,
    /// Log files (*.log, logs/)
    LogFile,
    /// Backup files (*~, *.bak, etc.)
    Backup,
    /// System junk (.DS_Store, Thumbs.db, etc.)
    SystemJunk,
    /// Package manager cache (npm, cargo, pip caches)
    PackageCache,
    /// Old/unused files (based on access time)
    OldFile,
    /// Duplicate file
    Duplicate,
}

impl JunkType {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            JunkType::BuildArtifact => "Build Artifact",
            JunkType::Cache => "Cache",
            JunkType::Temporary => "Temporary",
            JunkType::LogFile => "Log File",
            JunkType::Backup => "Backup",
            JunkType::SystemJunk => "System Junk",
            JunkType::PackageCache => "Package Cache",
            JunkType::OldFile => "Old File",
            JunkType::Duplicate => "Duplicate",
        }
    }

    /// Get icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            JunkType::BuildArtifact => "󰏗",
            JunkType::Cache => "󰃨",
            JunkType::Temporary => "󰃢",
            JunkType::LogFile => "󰌱",
            JunkType::Backup => "󰁯",
            JunkType::SystemJunk => "󰃤",
            JunkType::PackageCache => "󰏖",
            JunkType::OldFile => "󰔠",
            JunkType::Duplicate => "󰆑",
        }
    }

    /// Get priority for cleaning (higher = more safe to delete)
    pub fn priority(&self) -> u8 {
        match self {
            JunkType::Temporary => 100,
            JunkType::Cache => 90,
            JunkType::SystemJunk => 85,
            JunkType::LogFile => 80,
            JunkType::Backup => 70,
            JunkType::PackageCache => 60,
            JunkType::BuildArtifact => 50,
            JunkType::Duplicate => 40,
            JunkType::OldFile => 30,
        }
    }
}

/// Represents a file or directory entry with size information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    /// Entry name (file or directory name)
    pub name: String,

    /// Total size in bytes
    pub size: u64,

    /// Full path to the entry
    pub path: PathBuf,

    /// Whether this entry is a directory
    pub is_dir: bool,

    /// Child entries (only for directories)
    pub children: Vec<Entry>,

    /// Last modification time
    pub modified: Option<SystemTime>,

    /// Last access time (for age analysis)
    pub accessed: Option<SystemTime>,

    /// Total file count (recursive for directories)
    pub file_count: usize,

    /// Total directory count (recursive for directories)
    pub dir_count: usize,

    /// File category based on extension
    #[serde(default)]
    pub category: FileCategory,

    /// Junk type if this entry is detected as cleanable
    pub junk_type: Option<JunkType>,

    /// Content hash for duplicate detection (BLAKE3, skipped in serialization)
    #[serde(skip)]
    pub content_hash: Option<[u8; 32]>,

    /// Duplicate group ID (entries with same hash share this)
    #[serde(skip)]
    pub duplicate_group: Option<u64>,

    /// Error message if access was denied or other error occurred
    #[serde(skip)]
    pub error: Option<String>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            name: String::new(),
            size: 0,
            path: PathBuf::new(),
            is_dir: false,
            children: Vec::new(),
            modified: None,
            accessed: None,
            file_count: 0,
            dir_count: 0,
            category: FileCategory::Unknown,
            junk_type: None,
            content_hash: None,
            duplicate_group: None,
            error: None,
        }
    }
}

impl Entry {
    /// Create a new entry with the given path and name
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            name,
            path,
            ..Default::default()
        }
    }

    /// Get total items (files + directories)
    #[inline]
    pub fn total_items(&self) -> usize {
        self.file_count + self.dir_count
    }

    /// Check if entry has an error
    #[inline]
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Check if entry is empty (no children)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the percentage this entry takes of a parent size
    #[inline]
    pub fn percentage_of(&self, parent_size: u64) -> f64 {
        if parent_size == 0 {
            0.0
        } else {
            (self.size as f64 / parent_size as f64) * 100.0
        }
    }

    /// Sort children by size (descending)
    pub fn sort_by_size_desc(&mut self) {
        self.children.sort_by(|a, b| b.size.cmp(&a.size));
    }

    /// Sort children by name (ascending, case-insensitive)
    pub fn sort_by_name_asc(&mut self) {
        self.children
            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    /// Sort children by modification time (descending)
    pub fn sort_by_modified_desc(&mut self) {
        self.children.sort_by(|a, b| b.modified.cmp(&a.modified));
    }

    /// Sort children by file count (descending)
    pub fn sort_by_count_desc(&mut self) {
        self.children
            .sort_by(|a, b| b.file_count.cmp(&a.file_count));
    }

    /// Check if this entry is detected as junk
    #[inline]
    pub fn is_junk(&self) -> bool {
        self.junk_type.is_some()
    }

    /// Check if this entry is a duplicate
    #[inline]
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_group.is_some()
    }

    /// Get file extension if this is a file
    pub fn extension(&self) -> Option<&str> {
        if self.is_dir {
            None
        } else {
            self.path.extension().and_then(|e| e.to_str())
        }
    }

    /// Detect and set category from extension
    pub fn detect_category(&mut self) {
        if let Some(ext) = self.extension() {
            self.category = FileCategory::from_extension(ext);
        }
    }

    /// Get age in days since last modification
    pub fn age_days(&self) -> Option<u64> {
        self.modified.and_then(|m| {
            m.elapsed()
                .ok()
                .map(|d| d.as_secs() / (24 * 60 * 60))
        })
    }

    /// Get age in days since last access
    pub fn access_age_days(&self) -> Option<u64> {
        self.accessed.and_then(|a| {
            a.elapsed()
                .ok()
                .map(|d| d.as_secs() / (24 * 60 * 60))
        })
    }

    /// Recursively count junk entries and total junk size
    pub fn junk_stats(&self) -> (usize, u64) {
        let mut count = 0;
        let mut size = 0;

        if self.is_junk() {
            count += 1;
            size += self.size;
        }

        for child in &self.children {
            let (c, s) = child.junk_stats();
            count += c;
            size += s;
        }

        (count, size)
    }

    /// Recursively collect all junk entries
    pub fn collect_junk(&self) -> Vec<&Entry> {
        let mut junk = Vec::new();

        if self.is_junk() {
            junk.push(self);
        }

        for child in &self.children {
            junk.extend(child.collect_junk());
        }

        junk
    }

    /// Delete this entry from disk
    pub fn delete_from_disk(&self) -> std::io::Result<()> {
        if self.is_dir {
            std::fs::remove_dir_all(&self.path)
        } else {
            std::fs::remove_file(&self.path)
        }
    }

    /// Move this entry to system trash
    pub fn move_to_trash(&self) -> Result<(), trash::Error> {
        trash::delete(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_default() {
        let entry = Entry::default();
        assert!(entry.name.is_empty());
        assert_eq!(entry.size, 0);
        assert!(!entry.is_dir);
        assert!(entry.children.is_empty());
    }

    #[test]
    fn test_percentage_of() {
        let entry = Entry {
            size: 50,
            ..Default::default()
        };
        assert!((entry.percentage_of(100) - 50.0).abs() < 0.001);
        assert!((entry.percentage_of(0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_total_items() {
        let entry = Entry {
            file_count: 10,
            dir_count: 5,
            ..Default::default()
        };
        assert_eq!(entry.total_items(), 15);
    }
}
