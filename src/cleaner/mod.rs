//! Cleaner module for safe file deletion
//!
//! Provides both trash-based (recoverable) and permanent deletion with
//! progress tracking and batch operations.

mod batch;
mod safe_delete;

pub use batch::{BatchCleaner, CleaningProgress, CleaningResult};
pub use safe_delete::{delete_entry, delete_path, delete_path_with_size, DeleteError, DeleteResult};
