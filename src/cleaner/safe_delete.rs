//! Safe file deletion with trash and permanent options

use crate::config::DeletionMode;
use crate::scanner::Entry;
use std::path::Path;
use std::{fs, io};

/// Result type for deletion operations
pub type DeleteResult<T> = Result<T, DeleteError>;

/// Errors that can occur during deletion
#[derive(Debug)]
pub enum DeleteError {
    /// Path does not exist
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Path is protected and cannot be deleted
    Protected(String),
    /// Trash operation failed
    TrashError(String),
    /// IO error during permanent deletion
    IoError(io::Error),
    /// Operation was cancelled
    Cancelled,
}

impl std::fmt::Display for DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteError::NotFound(p) => write!(f, "Path not found: {}", p),
            DeleteError::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            DeleteError::Protected(p) => write!(f, "Protected path cannot be deleted: {}", p),
            DeleteError::TrashError(msg) => write!(f, "Trash error: {}", msg),
            DeleteError::IoError(e) => write!(f, "IO error: {}", e),
            DeleteError::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

impl std::error::Error for DeleteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DeleteError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DeleteError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => {
                DeleteError::NotFound(err.to_string())
            }
            io::ErrorKind::PermissionDenied => {
                DeleteError::PermissionDenied(err.to_string())
            }
            _ => DeleteError::IoError(err),
        }
    }
}

impl From<trash::Error> for DeleteError {
    fn from(err: trash::Error) -> Self {
        DeleteError::TrashError(err.to_string())
    }
}

/// Protected paths that should never be deleted
const PROTECTED_PATHS: &[&str] = &[
    "/",
    "/home",
    "/root",
    "/etc",
    "/usr",
    "/bin",
    "/sbin",
    "/lib",
    "/lib64",
    "/var",
    "/boot",
    "/dev",
    "/proc",
    "/sys",
    "/run",
    "/tmp",
    "/opt",
    // Common user directories
    ".ssh",
    ".gnupg",
    ".config",
    ".local",
];

/// Protected file names
const PROTECTED_NAMES: &[&str] = &[
    ".bashrc",
    ".zshrc",
    ".profile",
    ".bash_profile",
    ".gitconfig",
];

/// Check if a path is protected from deletion
pub fn is_protected(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check absolute protected paths
    for protected in PROTECTED_PATHS {
        if path_str == *protected {
            return true;
        }
    }

    // Check protected names
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();
        for protected_name in PROTECTED_NAMES {
            if name_str == *protected_name {
                return true;
            }
        }
    }

    false
}

/// Delete a path using the specified mode
pub fn delete_path(path: &Path, mode: DeletionMode) -> DeleteResult<u64> {
    // Check if path exists
    if !path.exists() {
        return Err(DeleteError::NotFound(path.display().to_string()));
    }

    // Check if path is protected
    if is_protected(path) {
        return Err(DeleteError::Protected(path.display().to_string()));
    }

    // Get size before deletion
    let size = get_size(path);

    // Perform deletion
    match mode {
        DeletionMode::Trash => {
            trash::delete(path)?;
        }
        DeletionMode::Permanent => {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(size)
}

/// Delete an Entry using the specified mode
pub fn delete_entry(entry: &Entry, mode: DeletionMode) -> DeleteResult<u64> {
    delete_path(&entry.path, mode)
}

/// Get the size of a path (file or directory)
fn get_size(path: &Path) -> u64 {
    if path.is_file() {
        path.metadata().map(|m| m.len()).unwrap_or(0)
    } else if path.is_dir() {
        calculate_dir_size(path)
    } else {
        0
    }
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(path: &Path) -> u64 {
    let mut size = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                size += entry_path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if entry_path.is_dir() {
                size += calculate_dir_size(&entry_path);
            }
        }
    }

    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_is_protected() {
        assert!(is_protected(Path::new("/")));
        assert!(is_protected(Path::new("/home")));
        assert!(is_protected(Path::new("/etc")));
        assert!(is_protected(Path::new(".ssh")));
        assert!(is_protected(Path::new(".bashrc")));

        assert!(!is_protected(Path::new("/tmp/test")));
        assert!(!is_protected(Path::new("/home/user/test.txt")));
        assert!(!is_protected(Path::new("node_modules")));
    }

    #[test]
    fn test_delete_file_permanent() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);

        assert!(file_path.exists());

        let size = delete_path(&file_path, DeletionMode::Permanent).unwrap();
        assert!(size > 0);
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_directory_permanent() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file_path = subdir.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);

        assert!(subdir.exists());

        let size = delete_path(&subdir, DeletionMode::Permanent).unwrap();
        assert!(size > 0);
        assert!(!subdir.exists());
    }

    #[test]
    fn test_delete_nonexistent() {
        let result = delete_path(Path::new("/nonexistent/path/file.txt"), DeletionMode::Permanent);
        assert!(matches!(result, Err(DeleteError::NotFound(_))));
    }

    #[test]
    fn test_delete_protected() {
        let result = delete_path(Path::new("/"), DeletionMode::Permanent);
        assert!(matches!(result, Err(DeleteError::Protected(_))));
    }
}
