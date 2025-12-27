//! Analyzer module for detecting junk, duplicates, and file analysis
//!
//! Provides functionality for:
//! - Junk file/directory detection based on patterns
//! - Duplicate file detection using content hashing
//! - File type categorization
//! - Age-based analysis

mod junk;
mod patterns;

pub use junk::{detect_junk, detect_junk_type, is_junk_directory, is_junk_file, JunkDetector};
pub use patterns::{JunkPattern, JunkPatterns, BUILTIN_PATTERNS};
