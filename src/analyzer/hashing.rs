//! Content hashing for duplicate detection using BLAKE3
//!
//! Uses a multi-stage approach for efficiency:
//! 1. Size filter - only compare files of the same size
//! 2. Partial hash - hash first 4KB for quick comparison
//! 3. Full hash - hash entire file for confirmation

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// Size of partial hash buffer (4KB)
const PARTIAL_HASH_SIZE: usize = 4096;

/// Buffer size for streaming hash (64KB)
const HASH_BUFFER_SIZE: usize = 65536;

/// Result of a hash operation
pub type HashResult<T> = Result<T, HashError>;

/// Errors that can occur during hashing
#[derive(Debug)]
pub enum HashError {
    /// File not found
    NotFound(String),
    /// IO error
    IoError(io::Error),
    /// File is a directory
    IsDirectory,
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashError::NotFound(p) => write!(f, "File not found: {}", p),
            HashError::IoError(e) => write!(f, "IO error: {}", e),
            HashError::IsDirectory => write!(f, "Cannot hash a directory"),
        }
    }
}

impl std::error::Error for HashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HashError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for HashError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => HashError::NotFound(err.to_string()),
            _ => HashError::IoError(err),
        }
    }
}

/// Compute a partial hash of the first N bytes of a file
/// This is used for quick comparison before computing full hash
pub fn partial_hash(path: &Path) -> HashResult<[u8; 32]> {
    if path.is_dir() {
        return Err(HashError::IsDirectory);
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; PARTIAL_HASH_SIZE];

    let bytes_read = reader.read(&mut buffer)?;

    let hash = blake3::hash(&buffer[..bytes_read]);
    Ok(*hash.as_bytes())
}

/// Compute the full content hash of a file
/// Uses streaming for memory efficiency with large files
pub fn full_hash(path: &Path) -> HashResult<[u8; 32]> {
    if path.is_dir() {
        return Err(HashError::IsDirectory);
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(*hash.as_bytes())
}

/// Hash a file, choosing partial or full based on size
/// For small files, full hash is used directly
/// For larger files, partial hash is more efficient for initial comparison
pub fn hash_file(path: &Path, use_partial: bool) -> HashResult<[u8; 32]> {
    if use_partial {
        partial_hash(path)
    } else {
        full_hash(path)
    }
}

/// Format a hash as a hex string
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Format a hash as a short hex string (first 8 chars)
pub fn hash_to_short_hex(hash: &[u8; 32]) -> String {
    hash[..4].iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_partial_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);

        let hash = partial_hash(&file_path).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_full_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);

        let hash = full_hash(&file_path).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_identical_files_same_hash() {
        let dir = tempdir().unwrap();

        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1)
            .unwrap()
            .write_all(b"identical content")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"identical content")
            .unwrap();

        let hash1 = full_hash(&file1).unwrap();
        let hash2 = full_hash(&file2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_files_different_hash() {
        let dir = tempdir().unwrap();

        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1)
            .unwrap()
            .write_all(b"content one")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"content two")
            .unwrap();

        let hash1 = full_hash(&file1).unwrap();
        let hash2 = full_hash(&file2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_to_hex() {
        let hash = [0u8; 32];
        let hex = hash_to_hex(&hash);
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c == '0'));
    }

    #[test]
    fn test_directory_error() {
        let dir = tempdir().unwrap();
        let result = partial_hash(dir.path());
        assert!(matches!(result, Err(HashError::IsDirectory)));
    }
}
