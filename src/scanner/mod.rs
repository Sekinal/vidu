//! Directory scanning with parallel processing
//!
//! This module provides functionality for scanning directories and building
//! a tree structure of entries with size information.

mod entry;
mod preview;
mod progress;
mod scan;

pub use entry::{Entry, FileCategory, JunkType};
pub use preview::{
    directory_preview, file_info_preview, read_file_preview, read_file_preview_with_limits,
    FilePreview,
};
pub use progress::{ProgressSummary, ScanProgress};
pub use scan::{refresh_children, scan_with_progress, ScanOptions};
