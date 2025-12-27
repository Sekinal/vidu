//! Entry struct representing a file or directory

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};

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

    /// Total file count (recursive for directories)
    pub file_count: usize,

    /// Total directory count (recursive for directories)
    pub dir_count: usize,

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
            file_count: 0,
            dir_count: 0,
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

    /// Delete this entry from disk
    pub fn delete_from_disk(&self) -> std::io::Result<()> {
        if self.is_dir {
            std::fs::remove_dir_all(&self.path)
        } else {
            std::fs::remove_file(&self.path)
        }
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
