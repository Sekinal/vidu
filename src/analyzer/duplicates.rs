//! Duplicate file detection
//!
//! Uses a multi-stage approach:
//! 1. Group files by size (duplicates must be same size)
//! 2. Compute partial hashes for quick comparison
//! 3. Compute full hashes to confirm duplicates

use super::hashing::{full_hash, partial_hash, HashError};
use crate::scanner::{Entry, JunkType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Minimum file size for duplicate detection (1KB default)
const DEFAULT_MIN_SIZE: u64 = 1024;

/// A group of duplicate files
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// Unique ID for this group
    pub id: u64,
    /// Content hash (BLAKE3)
    pub hash: [u8; 32],
    /// File size in bytes
    pub size: u64,
    /// Paths of duplicate files
    pub files: Vec<PathBuf>,
}

impl DuplicateGroup {
    /// Get the number of duplicates (files - 1)
    pub fn duplicate_count(&self) -> usize {
        self.files.len().saturating_sub(1)
    }

    /// Get the wasted space (size * (files - 1))
    pub fn wasted_space(&self) -> u64 {
        self.size * self.duplicate_count() as u64
    }

    /// Get the "original" file (oldest by modification time)
    /// Falls back to first file if times can't be compared
    pub fn original(&self) -> Option<&PathBuf> {
        self.files.first()
    }

    /// Get duplicate files (all except the original)
    pub fn duplicates(&self) -> &[PathBuf] {
        if self.files.len() > 1 {
            &self.files[1..]
        } else {
            &[]
        }
    }
}

/// Progress tracking for duplicate detection
#[derive(Debug)]
pub struct DuplicateProgress {
    /// Total files to process
    total_files: AtomicUsize,
    /// Files processed so far
    processed: AtomicUsize,
    /// Current phase description
    phase: std::sync::RwLock<String>,
    /// Whether cancelled
    cancelled: AtomicBool,
    /// Whether complete
    complete: AtomicBool,
}

impl DuplicateProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            total_files: AtomicUsize::new(0),
            processed: AtomicUsize::new(0),
            phase: std::sync::RwLock::new("Initializing".to_string()),
            cancelled: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        })
    }

    pub fn set_total(&self, total: usize) {
        self.total_files.store(total, Ordering::Relaxed);
    }

    pub fn inc_processed(&self) {
        self.processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn processed(&self) -> usize {
        self.processed.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> usize {
        self.total_files.load(Ordering::Relaxed)
    }

    pub fn set_phase(&self, phase: &str) {
        *self.phase.write().unwrap() = phase.to_string();
    }

    pub fn phase(&self) -> String {
        self.phase.read().unwrap().clone()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn mark_complete(&self) {
        self.complete.store(true, Ordering::Relaxed);
    }

    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    pub fn percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.processed() as f64 / total as f64) * 100.0
        }
    }
}

impl Default for DuplicateProgress {
    fn default() -> Self {
        Self {
            total_files: AtomicUsize::new(0),
            processed: AtomicUsize::new(0),
            phase: std::sync::RwLock::new("Initializing".to_string()),
            cancelled: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        }
    }
}

/// Result of duplicate detection
#[derive(Debug, Clone)]
pub struct DuplicateResult {
    /// Groups of duplicate files
    pub groups: Vec<DuplicateGroup>,
    /// Total files scanned
    pub files_scanned: usize,
    /// Total duplicates found
    pub duplicate_count: usize,
    /// Total wasted space
    pub wasted_space: u64,
}

impl DuplicateResult {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            files_scanned: 0,
            duplicate_count: 0,
            wasted_space: 0,
        }
    }

    /// Get groups sorted by wasted space (largest first)
    pub fn groups_by_wasted_space(&self) -> Vec<&DuplicateGroup> {
        let mut groups: Vec<_> = self.groups.iter().collect();
        groups.sort_by(|a, b| b.wasted_space().cmp(&a.wasted_space()));
        groups
    }
}

impl Default for DuplicateResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Duplicate file detector
pub struct DuplicateFinder {
    /// Minimum file size to consider
    min_size: u64,
    /// Skip hidden files
    skip_hidden: bool,
}

impl DuplicateFinder {
    pub fn new() -> Self {
        Self {
            min_size: DEFAULT_MIN_SIZE,
            skip_hidden: true,
        }
    }

    pub fn with_min_size(mut self, size: u64) -> Self {
        self.min_size = size;
        self
    }

    pub fn with_hidden(mut self, include: bool) -> Self {
        self.skip_hidden = !include;
        self
    }

    /// Find duplicates in an entry tree
    pub fn find_duplicates(&self, root: &Entry) -> DuplicateResult {
        self.find_duplicates_with_progress(root, None)
    }

    /// Find duplicates with progress tracking
    pub fn find_duplicates_with_progress(
        &self,
        root: &Entry,
        progress: Option<Arc<DuplicateProgress>>,
    ) -> DuplicateResult {
        let mut result = DuplicateResult::new();

        // Phase 1: Collect all files and group by size
        if let Some(ref p) = progress {
            p.set_phase("Grouping files by size");
        }

        let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        self.collect_files(root, &mut size_groups);

        // Filter to only sizes with multiple files
        let size_groups: HashMap<u64, Vec<PathBuf>> = size_groups
            .into_iter()
            .filter(|(size, files)| *size >= self.min_size && files.len() > 1)
            .collect();

        let total_files: usize = size_groups.values().map(|v| v.len()).sum();
        result.files_scanned = total_files;

        if let Some(ref p) = progress {
            p.set_total(total_files);
            p.set_phase("Computing hashes");
        }

        // Phase 2: Compute hashes and find duplicates
        let mut group_id: u64 = 0;

        for (size, files) in size_groups {
            if let Some(ref p) = progress {
                if p.is_cancelled() {
                    break;
                }
            }

            // For small groups, go straight to full hash
            // For larger groups, use partial hash first
            let use_partial = files.len() > 2 && size > 8192;

            let mut hash_groups: HashMap<[u8; 32], Vec<PathBuf>> = HashMap::new();

            for path in &files {
                if let Some(ref p) = progress {
                    if p.is_cancelled() {
                        break;
                    }
                    p.inc_processed();
                }

                let hash_result = if use_partial {
                    partial_hash(path)
                } else {
                    full_hash(path)
                };

                if let Ok(hash) = hash_result {
                    hash_groups.entry(hash).or_default().push(path.clone());
                }
            }

            // If we used partial hashes, verify with full hashes
            if use_partial {
                for (_, paths) in hash_groups.iter_mut() {
                    if paths.len() > 1 {
                        // Recompute with full hash
                        let mut full_hash_groups: HashMap<[u8; 32], Vec<PathBuf>> = HashMap::new();
                        for path in paths.iter() {
                            if let Ok(hash) = full_hash(path) {
                                full_hash_groups.entry(hash).or_default().push(path.clone());
                            }
                        }
                        // Keep only actual duplicates
                        for (hash, dup_paths) in full_hash_groups {
                            if dup_paths.len() > 1 {
                                result.groups.push(DuplicateGroup {
                                    id: group_id,
                                    hash,
                                    size,
                                    files: dup_paths,
                                });
                                group_id += 1;
                            }
                        }
                    }
                }
            } else {
                // Already using full hashes
                for (hash, paths) in hash_groups {
                    if paths.len() > 1 {
                        result.groups.push(DuplicateGroup {
                            id: group_id,
                            hash,
                            size,
                            files: paths,
                        });
                        group_id += 1;
                    }
                }
            }
        }

        // Calculate totals
        for group in &result.groups {
            result.duplicate_count += group.duplicate_count();
            result.wasted_space += group.wasted_space();
        }

        if let Some(ref p) = progress {
            p.mark_complete();
        }

        result
    }

    /// Recursively collect files from entry tree
    fn collect_files(&self, entry: &Entry, groups: &mut HashMap<u64, Vec<PathBuf>>) {
        if entry.is_dir {
            for child in &entry.children {
                self.collect_files(child, groups);
            }
        } else {
            // Skip hidden files if configured
            if self.skip_hidden && entry.name.starts_with('.') {
                return;
            }

            // Skip files that are too small
            if entry.size < self.min_size {
                return;
            }

            groups
                .entry(entry.size)
                .or_default()
                .push(entry.path.clone());
        }
    }

    /// Mark duplicates in an entry tree
    pub fn mark_duplicates(&self, root: &mut Entry) {
        let result = self.find_duplicates(root);

        // Create a map of path -> group_id for quick lookup
        let mut path_to_group: HashMap<PathBuf, u64> = HashMap::new();
        for group in &result.groups {
            for (i, path) in group.files.iter().enumerate() {
                // Skip the first file (original)
                if i > 0 {
                    path_to_group.insert(path.clone(), group.id);
                }
            }
        }

        // Mark entries
        self.mark_entries(root, &path_to_group);
    }

    fn mark_entries(&self, entry: &mut Entry, path_to_group: &HashMap<PathBuf, u64>) {
        if let Some(group_id) = path_to_group.get(&entry.path) {
            entry.junk_type = Some(JunkType::Duplicate);
            entry.duplicate_group = Some(*group_id);
        }

        for child in &mut entry.children {
            self.mark_entries(child, path_to_group);
        }
    }
}

impl Default for DuplicateFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_entry_tree(dir: &std::path::Path) -> Entry {
        use crate::scanner::{scan, ScanOptions};
        scan(
            dir.to_path_buf(),
            &ScanOptions::default().with_hidden(true),
        )
    }

    #[test]
    fn test_find_no_duplicates() {
        let dir = tempdir().unwrap();

        File::create(dir.path().join("file1.txt"))
            .unwrap()
            .write_all(b"unique content 1")
            .unwrap();
        File::create(dir.path().join("file2.txt"))
            .unwrap()
            .write_all(b"unique content 2")
            .unwrap();

        let entry = create_entry_tree(dir.path());
        let finder = DuplicateFinder::new().with_min_size(1);
        let result = finder.find_duplicates(&entry);

        assert_eq!(result.groups.len(), 0);
        assert_eq!(result.duplicate_count, 0);
    }

    #[test]
    fn test_find_duplicates() {
        let dir = tempdir().unwrap();

        // Create three files with identical content
        let content = b"this is duplicate content that is long enough";
        File::create(dir.path().join("file1.txt"))
            .unwrap()
            .write_all(content)
            .unwrap();
        File::create(dir.path().join("file2.txt"))
            .unwrap()
            .write_all(content)
            .unwrap();
        File::create(dir.path().join("file3.txt"))
            .unwrap()
            .write_all(content)
            .unwrap();

        let entry = create_entry_tree(dir.path());
        let finder = DuplicateFinder::new().with_min_size(1);
        let result = finder.find_duplicates(&entry);

        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].files.len(), 3);
        assert_eq!(result.groups[0].duplicate_count(), 2);
    }

    #[test]
    fn test_wasted_space() {
        let dir = tempdir().unwrap();

        let content = b"test content for wasted space calculation!!!!";
        let size = content.len() as u64;

        File::create(dir.path().join("file1.txt"))
            .unwrap()
            .write_all(content)
            .unwrap();
        File::create(dir.path().join("file2.txt"))
            .unwrap()
            .write_all(content)
            .unwrap();

        let entry = create_entry_tree(dir.path());
        let finder = DuplicateFinder::new().with_min_size(1);
        let result = finder.find_duplicates(&entry);

        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].wasted_space(), size); // 1 duplicate
        assert_eq!(result.wasted_space, size);
    }

    #[test]
    fn test_skip_small_files() {
        let dir = tempdir().unwrap();

        // Create small duplicate files
        File::create(dir.path().join("small1.txt"))
            .unwrap()
            .write_all(b"hi")
            .unwrap();
        File::create(dir.path().join("small2.txt"))
            .unwrap()
            .write_all(b"hi")
            .unwrap();

        let entry = create_entry_tree(dir.path());
        let finder = DuplicateFinder::new().with_min_size(100);
        let result = finder.find_duplicates(&entry);

        // Should skip because files are too small
        assert_eq!(result.groups.len(), 0);
    }
}
