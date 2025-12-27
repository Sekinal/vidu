//! Directory scanning with parallel processing

use super::{Entry, ScanProgress};
use crate::constants::scanner::{MIN_ENTRIES_FOR_PARALLEL, PARALLEL_SCAN_DEPTH, PROGRESS_UPDATE_DEPTH};
use rayon::prelude::*;
use std::{fs, path::PathBuf, sync::Arc};

/// Virtual/pseudo filesystems that should be skipped
/// These don't represent real disk usage
const VIRTUAL_FILESYSTEMS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/snap",
];

/// Specific paths that report fake sizes
const SKIP_PATHS: &[&str] = &[
    "/proc/kcore",      // Reports ~128TB (kernel virtual memory)
    "/sys/kernel",
];

/// Check if a path is a virtual filesystem that should be skipped
fn is_virtual_filesystem(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy();

    // Check exact virtual filesystem roots
    for vfs in VIRTUAL_FILESYSTEMS {
        if path_str == *vfs || path_str.starts_with(&format!("{}/", vfs)) {
            return true;
        }
    }

    // Check specific skip paths
    for skip in SKIP_PATHS {
        if path_str == *skip {
            return true;
        }
    }

    false
}

/// Options for scanning
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Whether to show hidden files
    pub show_hidden: bool,

    /// Maximum depth to scan (None for unlimited)
    pub max_depth: Option<usize>,

    /// Whether to follow symlinks
    pub follow_symlinks: bool,

    /// Whether to skip virtual filesystems (/proc, /sys, /dev, etc.)
    pub skip_virtual: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            max_depth: None,
            follow_symlinks: false,
            skip_virtual: true,
        }
    }
}

impl ScanOptions {
    /// Create options with hidden files shown
    pub fn with_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Create options with virtual filesystem skipping
    pub fn with_skip_virtual(mut self, skip: bool) -> Self {
        self.skip_virtual = skip;
        self
    }
}

/// Scan a path and build the entry tree
pub fn scan(path: PathBuf, options: &ScanOptions) -> Entry {
    scan_with_progress(path, options, None)
}

/// Scan with progress tracking
pub fn scan_with_progress(
    path: PathBuf,
    options: &ScanOptions,
    progress: Option<Arc<ScanProgress>>,
) -> Entry {
    scan_internal(path, options, &progress, 0)
}

fn scan_internal(
    path: PathBuf,
    options: &ScanOptions,
    progress: &Option<Arc<ScanProgress>>,
    depth: usize,
) -> Entry {
    // Check for cancellation
    if let Some(p) = progress {
        if p.is_cancelled() {
            return Entry::default();
        }
    }

    // Check max depth
    if let Some(max) = options.max_depth {
        if depth > max {
            return Entry::default();
        }
    }

    // Skip virtual filesystems
    if options.skip_virtual && is_virtual_filesystem(&path) {
        let name = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("."))
            .to_string_lossy()
            .to_string();
        let mut entry = Entry::new(path.clone(), name);
        entry.is_dir = true;
        entry.error = Some("Virtual filesystem (skipped)".to_string());
        return entry;
    }

    let metadata = fs::symlink_metadata(&path);
    let name = path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("."))
        .to_string_lossy()
        .to_string();

    let mut entry = Entry::new(path.clone(), name);

    let meta = match metadata {
        Ok(m) => m,
        Err(e) => {
            entry.error = Some(format!("Access denied: {}", e));
            return entry;
        }
    };

    entry.modified = meta.modified().ok();

    // Handle symlinks - don't follow them to avoid infinite loops
    if meta.is_symlink() && !options.follow_symlinks {
        entry.size = meta.len();
        entry.file_count = 1;
        return entry;
    }

    if meta.is_dir() {
        entry.is_dir = true;
        entry.dir_count = 1;

        // Update progress
        if let Some(p) = progress {
            p.inc_dirs();
            if depth < PROGRESS_UPDATE_DEPTH {
                p.set_current(&path.to_string_lossy());
            }
        }

        // Read directory entries
        let entries: Vec<_> = match fs::read_dir(&path) {
            Ok(rd) => rd
                .filter_map(|r| r.ok())
                .filter(|e| {
                    // Filter hidden files
                    if !options.show_hidden && e.file_name().to_string_lossy().starts_with('.') {
                        return false;
                    }
                    // Filter virtual filesystems
                    if options.skip_virtual && is_virtual_filesystem(&e.path()) {
                        return false;
                    }
                    true
                })
                .collect(),
            Err(e) => {
                entry.error = Some(format!("Cannot read directory: {}", e));
                return entry;
            }
        };

        // Parallel scan for children (only at shallow depths to avoid too many threads)
        if depth < PARALLEL_SCAN_DEPTH && entries.len() >= MIN_ENTRIES_FOR_PARALLEL {
            entry.children = entries
                .par_iter()
                .map(|dir_entry| scan_internal(dir_entry.path(), options, progress, depth + 1))
                .collect();
        } else {
            entry.children = entries
                .iter()
                .map(|dir_entry| scan_internal(dir_entry.path(), options, progress, depth + 1))
                .collect();
        }

        // Aggregate stats
        for child in &entry.children {
            entry.size += child.size;
            entry.file_count += child.file_count;
            entry.dir_count += child.dir_count;
        }

        // Add directory's own size
        entry.size += meta.len();

        // Sort by size descending by default
        entry.sort_by_size_desc();
    } else {
        // File
        entry.size = meta.len();
        entry.file_count = 1;

        if let Some(p) = progress {
            p.inc_files();
            p.add_bytes(entry.size);
        }
    }

    entry
}

/// Refresh only a directory's children (faster than full rescan)
pub fn refresh_children(entry: &mut Entry, options: &ScanOptions) {
    if !entry.is_dir {
        return;
    }

    let fresh = scan(entry.path.clone(), options);
    entry.children = fresh.children;
    entry.size = fresh.size;
    entry.file_count = fresh.file_count;
    entry.dir_count = fresh.dir_count;
    entry.error = fresh.error;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempdir().unwrap();
        let options = ScanOptions::default();
        let entry = scan(dir.path().to_path_buf(), &options);

        assert!(entry.is_dir);
        assert!(entry.children.is_empty());
        assert_eq!(entry.file_count, 0);
    }

    #[test]
    fn test_scan_with_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let options = ScanOptions::default();
        let entry = scan(dir.path().to_path_buf(), &options);

        assert!(entry.is_dir);
        assert_eq!(entry.children.len(), 1);
        assert_eq!(entry.file_count, 1);
        assert_eq!(entry.children[0].name, "test.txt");
    }

    #[test]
    fn test_hidden_files() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join(".hidden")).unwrap();
        File::create(dir.path().join("visible")).unwrap();

        // Without hidden
        let options = ScanOptions::default();
        let entry = scan(dir.path().to_path_buf(), &options);
        assert_eq!(entry.children.len(), 1);

        // With hidden
        let options = ScanOptions::default().with_hidden(true);
        let entry = scan(dir.path().to_path_buf(), &options);
        assert_eq!(entry.children.len(), 2);
    }
}
