//! Analyzer module for detecting junk, duplicates, and file analysis
//!
//! Provides functionality for:
//! - Junk file/directory detection based on patterns
//! - Duplicate file detection using content hashing
//! - File type categorization
//! - Age-based analysis
//! - Large file detection
//! - Package manager and browser cache detection
//! - Cleaning presets

mod age;
mod caches;
mod duplicates;
mod file_types;
mod hashing;
mod junk;
mod large_files;
mod patterns;
mod presets;

pub use age::{age_in_days, format_age, AgeBucket, AgeBucketStats, AgeAnalysis, OldFile};
pub use caches::{CacheCategory, CacheDetector, CacheLocation};
pub use duplicates::{DuplicateFinder, DuplicateGroup, DuplicateProgress, DuplicateResult};
pub use file_types::{CategoryStats, FileTypeAnalysis};
pub use hashing::{full_hash, hash_to_hex, hash_to_short_hex, partial_hash, HashError, HashResult};
pub use junk::{detect_junk, detect_junk_type, is_junk_directory, is_junk_file, JunkDetector, JunkStats};
pub use large_files::{find_files_over, find_large_files, thresholds, LargeFile, LargeFileFinder};
pub use patterns::{JunkPattern, JunkPatterns, BUILTIN_PATTERNS};
pub use presets::{BuiltinPresets, CleaningPreset};
