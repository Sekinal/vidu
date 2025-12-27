//! Large file detection
//!
//! Maintains a list of the largest files for quick access

use crate::scanner::{Entry, FileCategory, JunkType};
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::time::SystemTime;

/// A large file entry
#[derive(Debug, Clone)]
pub struct LargeFile {
    /// Path to the file
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// File category
    pub category: FileCategory,
    /// Whether it's a junk file
    pub junk_type: Option<JunkType>,
    /// Last modified time
    pub modified: Option<SystemTime>,
}

impl LargeFile {
    fn from_entry(entry: &Entry) -> Self {
        Self {
            path: entry.path.clone(),
            size: entry.size,
            category: entry.category,
            junk_type: entry.junk_type,
            modified: entry.modified,
        }
    }
}

// For BinaryHeap - we want min-heap to efficiently track top N
impl PartialEq for LargeFile {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for LargeFile {}

impl PartialOrd for LargeFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LargeFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap behavior
        other.size.cmp(&self.size)
    }
}

/// Large file finder that maintains top N files
#[derive(Debug)]
pub struct LargeFileFinder {
    /// Maximum number of files to track
    max_files: usize,
    /// Minimum size threshold
    min_size: u64,
    /// Heap of large files (min-heap for efficient replacement)
    files: BinaryHeap<LargeFile>,
}

impl LargeFileFinder {
    /// Create a new finder tracking top N files
    pub fn new(max_files: usize) -> Self {
        Self {
            max_files,
            min_size: 0,
            files: BinaryHeap::new(),
        }
    }

    /// Set minimum size threshold
    pub fn with_min_size(mut self, min_size: u64) -> Self {
        self.min_size = min_size;
        self
    }

    /// Add a file if it qualifies as large
    pub fn add(&mut self, file: LargeFile) {
        if file.size < self.min_size {
            return;
        }

        if self.files.len() < self.max_files {
            self.files.push(file);
        } else if let Some(smallest) = self.files.peek() {
            if file.size > smallest.size {
                self.files.pop();
                self.files.push(file);
            }
        }
    }

    /// Add an entry if it's a file
    pub fn add_entry(&mut self, entry: &Entry) {
        if !entry.is_dir {
            self.add(LargeFile::from_entry(entry));
        }
    }

    /// Process an entire entry tree
    pub fn process_tree(&mut self, entry: &Entry) {
        if entry.is_dir {
            for child in &entry.children {
                self.process_tree(child);
            }
        } else {
            self.add_entry(entry);
        }
    }

    /// Get the large files sorted by size (largest first)
    pub fn get_files(&self) -> Vec<LargeFile> {
        let mut files: Vec<_> = self.files.iter().cloned().collect();
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files
    }

    /// Get total size of all tracked files
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Get count of tracked files
    pub fn count(&self) -> usize {
        self.files.len()
    }

    /// Check if we have any files
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get the smallest file in our list (threshold for inclusion)
    pub fn min_tracked_size(&self) -> Option<u64> {
        self.files.peek().map(|f| f.size)
    }
}

impl Default for LargeFileFinder {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Find the N largest files in an entry tree
pub fn find_large_files(entry: &Entry, n: usize) -> Vec<LargeFile> {
    let mut finder = LargeFileFinder::new(n);
    finder.process_tree(entry);
    finder.get_files()
}

/// Find files larger than a threshold
pub fn find_files_over(entry: &Entry, min_size: u64, max_results: usize) -> Vec<LargeFile> {
    let mut finder = LargeFileFinder::new(max_results).with_min_size(min_size);
    finder.process_tree(entry);
    finder.get_files()
}

/// Size thresholds for common sizes
pub mod thresholds {
    /// 1 MB
    pub const MB_1: u64 = 1024 * 1024;
    /// 10 MB
    pub const MB_10: u64 = 10 * 1024 * 1024;
    /// 100 MB
    pub const MB_100: u64 = 100 * 1024 * 1024;
    /// 500 MB
    pub const MB_500: u64 = 500 * 1024 * 1024;
    /// 1 GB
    pub const GB_1: u64 = 1024 * 1024 * 1024;
    /// 5 GB
    pub const GB_5: u64 = 5 * 1024 * 1024 * 1024;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(name: &str, size: u64) -> Entry {
        Entry {
            name: name.to_string(),
            path: PathBuf::from(name),
            size,
            modified: Some(SystemTime::now()),
            ..Default::default()
        }
    }

    #[test]
    fn test_large_file_finder() {
        let mut finder = LargeFileFinder::new(3);

        finder.add_entry(&create_test_entry("small.txt", 100));
        finder.add_entry(&create_test_entry("medium.txt", 500));
        finder.add_entry(&create_test_entry("large.txt", 1000));
        finder.add_entry(&create_test_entry("huge.txt", 2000));
        finder.add_entry(&create_test_entry("tiny.txt", 50));

        let files = finder.get_files();

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].size, 2000); // huge
        assert_eq!(files[1].size, 1000); // large
        assert_eq!(files[2].size, 500); // medium
    }

    #[test]
    fn test_min_size_threshold() {
        let mut finder = LargeFileFinder::new(10).with_min_size(500);

        finder.add_entry(&create_test_entry("small.txt", 100));
        finder.add_entry(&create_test_entry("medium.txt", 500));
        finder.add_entry(&create_test_entry("large.txt", 1000));

        let files = finder.get_files();

        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.size >= 500));
    }

    #[test]
    fn test_process_tree() {
        let root = Entry {
            name: "root".to_string(),
            path: PathBuf::from("root"),
            is_dir: true,
            children: vec![
                create_test_entry("a.txt", 100),
                create_test_entry("b.txt", 200),
                Entry {
                    name: "subdir".to_string(),
                    path: PathBuf::from("subdir"),
                    is_dir: true,
                    children: vec![create_test_entry("c.txt", 300)],
                    modified: Some(SystemTime::now()),
                    ..Default::default()
                },
            ],
            modified: Some(SystemTime::now()),
            ..Default::default()
        };

        let files = find_large_files(&root, 10);

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].size, 300); // largest
    }

    #[test]
    fn test_find_files_over() {
        let root = Entry {
            name: "root".to_string(),
            path: PathBuf::from("root"),
            is_dir: true,
            children: vec![
                create_test_entry("small.txt", 100),
                create_test_entry("large.txt", 1000),
            ],
            modified: Some(SystemTime::now()),
            ..Default::default()
        };

        let files = find_files_over(&root, 500, 10);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size, 1000);
    }
}
