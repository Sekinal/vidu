//! Application error types

use std::path::PathBuf;
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum ViduError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path not found: {path}")]
    PathNotFound { path: PathBuf },

    #[error("access denied: {path}")]
    AccessDenied { path: PathBuf },

    #[error("failed to read directory: {path}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("scan cancelled")]
    ScanCancelled,
}

/// Cache-specific errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("cache not found for path: {0}")]
    NotFound(PathBuf),

    #[error("cache expired for path: {0}")]
    Expired(PathBuf),

    #[error("failed to serialize cache data")]
    Serialization(#[source] bincode::error::EncodeError),

    #[error("failed to deserialize cache data")]
    Deserialization(#[source] bincode::error::DecodeError),

    #[error("failed to compress cache data: {0}")]
    Compression(String),

    #[error("failed to decompress cache data: {0}")]
    Decompression(String),

    #[error("cache I/O error")]
    Io(#[from] std::io::Error),

    #[error("could not determine cache directory")]
    NoCacheDir,
}

/// Scanner-specific errors
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("access denied: {path}")]
    AccessDenied {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read directory: {path}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("scan cancelled by user")]
    Cancelled,

    #[error("symlink loop detected at: {0}")]
    SymlinkLoop(PathBuf),
}

/// Result type alias for ViduError
pub type Result<T> = std::result::Result<T, ViduError>;

/// Result type alias for CacheError
pub type CacheResult<T> = std::result::Result<T, CacheError>;

/// Result type alias for ScanError
pub type ScanResult<T> = std::result::Result<T, ScanError>;
