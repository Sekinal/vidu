//! Batch file deletion with progress tracking

use super::safe_delete::{delete_path, DeleteError, DeleteResult};
use crate::config::DeletionMode;
use crate::scanner::Entry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Result of a cleaning operation
#[derive(Debug, Clone)]
pub struct CleaningResult {
    /// Total number of items processed
    pub total_items: usize,
    /// Number of successfully deleted items
    pub deleted_count: usize,
    /// Number of failed deletions
    pub failed_count: usize,
    /// Total bytes freed
    pub bytes_freed: u64,
    /// Errors that occurred
    pub errors: Vec<(PathBuf, String)>,
}

impl CleaningResult {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            deleted_count: 0,
            failed_count: 0,
            bytes_freed: 0,
            errors: Vec::new(),
        }
    }

    /// Add a successful deletion
    pub fn add_success(&mut self, size: u64) {
        self.deleted_count += 1;
        self.total_items += 1;
        self.bytes_freed += size;
    }

    /// Add a failed deletion
    pub fn add_failure(&mut self, path: PathBuf, error: String) {
        self.failed_count += 1;
        self.total_items += 1;
        self.errors.push((path, error));
    }

    /// Check if all operations succeeded
    pub fn is_success(&self) -> bool {
        self.failed_count == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_items == 0 {
            100.0
        } else {
            (self.deleted_count as f64 / self.total_items as f64) * 100.0
        }
    }
}

impl Default for CleaningResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress tracking for batch operations
#[derive(Debug)]
pub struct CleaningProgress {
    /// Total items to process
    total: AtomicUsize,
    /// Items processed so far
    processed: AtomicUsize,
    /// Bytes freed so far
    bytes_freed: AtomicU64,
    /// Current item being processed
    current_path: std::sync::RwLock<String>,
    /// Whether the operation has been cancelled
    cancelled: AtomicBool,
    /// Whether the operation is complete
    complete: AtomicBool,
}

impl CleaningProgress {
    pub fn new(total: usize) -> Arc<Self> {
        Arc::new(Self {
            total: AtomicUsize::new(total),
            processed: AtomicUsize::new(0),
            bytes_freed: AtomicU64::new(0),
            current_path: std::sync::RwLock::new(String::new()),
            cancelled: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        })
    }

    /// Get total items to process
    pub fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    /// Get number of processed items
    pub fn processed(&self) -> usize {
        self.processed.load(Ordering::Relaxed)
    }

    /// Get bytes freed
    pub fn bytes_freed(&self) -> u64 {
        self.bytes_freed.load(Ordering::Relaxed)
    }

    /// Get current item path
    pub fn current(&self) -> String {
        self.current_path.read().unwrap().clone()
    }

    /// Get progress as percentage
    pub fn percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            100.0
        } else {
            (self.processed() as f64 / total as f64) * 100.0
        }
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Mark as complete
    fn mark_complete(&self) {
        self.complete.store(true, Ordering::Relaxed);
    }

    /// Update progress
    fn update(&self, path: &str, bytes: u64) {
        self.processed.fetch_add(1, Ordering::Relaxed);
        self.bytes_freed.fetch_add(bytes, Ordering::Relaxed);
        *self.current_path.write().unwrap() = path.to_string();
    }
}

/// Batch cleaner for multiple items
pub struct BatchCleaner {
    /// Deletion mode
    mode: DeletionMode,
    /// Items to delete
    items: Vec<PathBuf>,
}

impl BatchCleaner {
    /// Create a new batch cleaner
    pub fn new(mode: DeletionMode) -> Self {
        Self {
            mode,
            items: Vec::new(),
        }
    }

    /// Add a path to delete
    pub fn add_path(&mut self, path: PathBuf) {
        self.items.push(path);
    }

    /// Add an entry to delete
    pub fn add_entry(&mut self, entry: &Entry) {
        self.items.push(entry.path.clone());
    }

    /// Add multiple entries to delete
    pub fn add_entries(&mut self, entries: &[&Entry]) {
        for entry in entries {
            self.items.push(entry.path.clone());
        }
    }

    /// Get number of items to delete
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Execute the batch deletion without progress tracking
    pub fn execute(&self) -> CleaningResult {
        let mut result = CleaningResult::new();

        for path in &self.items {
            match delete_path(path, self.mode) {
                Ok(size) => result.add_success(size),
                Err(e) => result.add_failure(path.clone(), e.to_string()),
            }
        }

        result
    }

    /// Execute the batch deletion with progress tracking
    pub fn execute_with_progress(&self, progress: Arc<CleaningProgress>) -> CleaningResult {
        let mut result = CleaningResult::new();

        for path in &self.items {
            // Check for cancellation
            if progress.is_cancelled() {
                break;
            }

            let path_str = path.display().to_string();

            match delete_path(path, self.mode) {
                Ok(size) => {
                    result.add_success(size);
                    progress.update(&path_str, size);
                }
                Err(e) => {
                    result.add_failure(path.clone(), e.to_string());
                    progress.update(&path_str, 0);
                }
            }
        }

        progress.mark_complete();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_cleaning_result() {
        let mut result = CleaningResult::new();

        result.add_success(1000);
        result.add_success(2000);
        result.add_failure(PathBuf::from("/failed"), "error".to_string());

        assert_eq!(result.total_items, 3);
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.bytes_freed, 3000);
        assert!(!result.is_success());
    }

    #[test]
    fn test_batch_cleaner() {
        let dir = tempdir().unwrap();

        // Create some test files
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1).unwrap().write_all(b"test1").unwrap();
        File::create(&file2).unwrap().write_all(b"test2").unwrap();

        let mut cleaner = BatchCleaner::new(DeletionMode::Permanent);
        cleaner.add_path(file1.clone());
        cleaner.add_path(file2.clone());

        assert_eq!(cleaner.count(), 2);

        let result = cleaner.execute();

        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.failed_count, 0);
        assert!(!file1.exists());
        assert!(!file2.exists());
    }

    #[test]
    fn test_batch_cleaner_with_progress() {
        let dir = tempdir().unwrap();

        let file1 = dir.path().join("file1.txt");
        File::create(&file1).unwrap().write_all(b"test").unwrap();

        let mut cleaner = BatchCleaner::new(DeletionMode::Permanent);
        cleaner.add_path(file1.clone());

        let progress = CleaningProgress::new(1);
        let result = cleaner.execute_with_progress(progress.clone());

        assert_eq!(result.deleted_count, 1);
        assert!(progress.is_complete());
        assert_eq!(progress.processed(), 1);
    }

    #[test]
    fn test_progress_cancellation() {
        let progress = CleaningProgress::new(10);
        assert!(!progress.is_cancelled());

        progress.cancel();
        assert!(progress.is_cancelled());
    }
}
