//! Scan progress tracking

use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, RwLock,
};

/// Progress tracking during directory scan
#[derive(Default)]
pub struct ScanProgress {
    /// Number of files scanned
    pub files_scanned: AtomicUsize,

    /// Number of directories scanned
    pub dirs_scanned: AtomicUsize,

    /// Total bytes scanned
    pub bytes_scanned: AtomicU64,

    /// Current path being scanned
    pub current_path: RwLock<String>,

    /// Whether the scan is complete
    pub is_complete: AtomicBool,

    /// Whether the scan was cancelled
    pub cancelled: AtomicBool,
}

impl ScanProgress {
    /// Create a new progress tracker wrapped in Arc
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Get the number of files scanned
    #[inline]
    pub fn files(&self) -> usize {
        self.files_scanned.load(Ordering::Relaxed)
    }

    /// Get the number of directories scanned
    #[inline]
    pub fn dirs(&self) -> usize {
        self.dirs_scanned.load(Ordering::Relaxed)
    }

    /// Get the total bytes scanned
    #[inline]
    pub fn bytes(&self) -> u64 {
        self.bytes_scanned.load(Ordering::Relaxed)
    }

    /// Get the current path being scanned
    pub fn current(&self) -> String {
        self.current_path
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Cancel the scan
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if the scan was cancelled
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Check if the scan is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.is_complete.load(Ordering::SeqCst)
    }

    /// Mark the scan as complete
    pub fn mark_complete(&self) {
        self.is_complete.store(true, Ordering::SeqCst);
    }

    /// Increment the file count
    #[inline]
    pub fn inc_files(&self) {
        self.files_scanned.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the directory count
    #[inline]
    pub fn inc_dirs(&self) {
        self.dirs_scanned.fetch_add(1, Ordering::Relaxed);
    }

    /// Add to the byte count
    #[inline]
    pub fn add_bytes(&self, bytes: u64) {
        self.bytes_scanned.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Update the current path
    pub fn set_current(&self, path: &str) {
        if let Ok(mut current) = self.current_path.write() {
            *current = path.to_string();
        }
    }

    /// Get a summary of the progress
    pub fn summary(&self) -> ProgressSummary {
        ProgressSummary {
            files: self.files(),
            dirs: self.dirs(),
            bytes: self.bytes(),
            is_complete: self.is_complete(),
            is_cancelled: self.is_cancelled(),
        }
    }
}

/// Summary of scan progress
#[derive(Debug, Clone, Copy)]
pub struct ProgressSummary {
    pub files: usize,
    pub dirs: usize,
    pub bytes: u64,
    pub is_complete: bool,
    pub is_cancelled: bool,
}

impl ProgressSummary {
    /// Get total items scanned
    #[inline]
    pub fn total_items(&self) -> usize {
        self.files + self.dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracking() {
        let progress = ScanProgress::new();

        progress.inc_files();
        progress.inc_files();
        progress.inc_dirs();
        progress.add_bytes(1000);

        assert_eq!(progress.files(), 2);
        assert_eq!(progress.dirs(), 1);
        assert_eq!(progress.bytes(), 1000);
    }

    #[test]
    fn test_cancellation() {
        let progress = ScanProgress::new();
        assert!(!progress.is_cancelled());

        progress.cancel();
        assert!(progress.is_cancelled());
    }
}
