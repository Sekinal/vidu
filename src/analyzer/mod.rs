//! Analyzer module for detecting junk, duplicates, and file analysis
//!
//! Provides functionality for:
//! - Junk file/directory detection based on patterns
//! - Duplicate file detection using content hashing
//! - File type categorization
//! - Age-based analysis
//! - Package manager and browser cache detection

mod caches;
mod duplicates;
mod hashing;
mod junk;
mod patterns;

pub use caches::{CacheCategory, CacheDetector, CacheLocation};
pub use duplicates::{DuplicateFinder, DuplicateGroup, DuplicateProgress, DuplicateResult};
pub use hashing::{full_hash, hash_to_hex, hash_to_short_hex, partial_hash, HashError, HashResult};
pub use junk::{detect_junk, detect_junk_type, is_junk_directory, is_junk_file, JunkDetector, JunkStats};
pub use patterns::{JunkPattern, JunkPatterns, BUILTIN_PATTERNS};
