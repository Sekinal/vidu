//! Cache management for scan results
//!
//! Provides functionality to save and load scan results from disk,
//! reducing the need for repeated scans of the same directories.

use crate::constants::cache::{EXPIRY_DURATION, FILE_EXTENSION, ORGANIZATION, QUALIFIER};
use crate::error::{CacheError, CacheResult};
use crate::scanner::Entry;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Cache entry metadata
#[derive(Serialize, Deserialize, Debug)]
struct CacheMetadata {
    /// When the cache was created
    created: SystemTime,

    /// Original path that was scanned
    path: PathBuf,

    /// Cache format version
    version: u32,
}

impl CacheMetadata {
    const CURRENT_VERSION: u32 = 1;

    fn new(path: PathBuf) -> Self {
        Self {
            created: SystemTime::now(),
            path,
            version: Self::CURRENT_VERSION,
        }
    }

    fn is_expired(&self, max_age: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.created)
            .map(|age| age > max_age)
            .unwrap_or(true)
    }

    fn is_compatible(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }
}

/// Cached scan data
#[derive(Serialize, Deserialize)]
struct CacheData {
    metadata: CacheMetadata,
    entry: Entry,
}

/// Cache manager for scan results
#[derive(Clone)]
pub struct CacheManager {
    cache_dir: PathBuf,
    max_age: Duration,
    use_compression: bool,
}

impl CacheManager {
    /// Create a new cache manager with default settings
    pub fn new() -> CacheResult<Self> {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, ORGANIZATION)
            .ok_or(CacheError::NoCacheDir)?;

        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            cache_dir,
            max_age: EXPIRY_DURATION,
            use_compression: true,
        })
    }

    /// Set the maximum age for cache entries
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Enable or disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.use_compression = enabled;
        self
    }

    /// Get the cache file path for a given scan path
    fn cache_file(&self, path: &Path) -> PathBuf {
        let hash = md5::compute(path.to_string_lossy().as_bytes());
        self.cache_dir.join(format!("{:x}.{}", hash, FILE_EXTENSION))
    }

    /// Save a scan result to the cache
    pub fn save(&self, entry: &Entry) -> CacheResult<()> {
        let cache_file = self.cache_file(&entry.path);
        let data = CacheData {
            metadata: CacheMetadata::new(entry.path.clone()),
            entry: entry.clone(),
        };

        let config = bincode::config::standard();
        let serialized =
            bincode::serde::encode_to_vec(&data, config).map_err(CacheError::Serialization)?;

        let final_data = if self.use_compression {
            lz4_flex::compress_prepend_size(&serialized)
        } else {
            serialized
        };

        fs::write(cache_file, final_data)?;
        Ok(())
    }

    /// Load a scan result from the cache
    pub fn load(&self, path: &Path) -> CacheResult<Entry> {
        let cache_file = self.cache_file(path);

        if !cache_file.exists() {
            return Err(CacheError::NotFound(path.to_path_buf()));
        }

        let raw_data = fs::read(&cache_file)?;

        let decompressed = if self.use_compression {
            lz4_flex::decompress_size_prepended(&raw_data)
                .map_err(|e| CacheError::Decompression(e.to_string()))?
        } else {
            raw_data
        };

        let config = bincode::config::standard();
        let (data, _): (CacheData, usize) =
            bincode::serde::decode_from_slice(&decompressed, config)
                .map_err(CacheError::Deserialization)?;

        // Check version compatibility
        if !data.metadata.is_compatible() {
            // Remove incompatible cache
            let _ = fs::remove_file(&cache_file);
            return Err(CacheError::NotFound(path.to_path_buf()));
        }

        // Check expiry
        if data.metadata.is_expired(self.max_age) {
            // Remove expired cache
            let _ = fs::remove_file(&cache_file);
            return Err(CacheError::Expired(path.to_path_buf()));
        }

        Ok(data.entry)
    }

    /// Check if a valid cache exists for the given path
    pub fn exists(&self, path: &Path) -> bool {
        self.load(path).is_ok()
    }

    /// Remove the cache for a given path
    pub fn remove(&self, path: &Path) -> CacheResult<()> {
        let cache_file = self.cache_file(path);
        if cache_file.exists() {
            fs::remove_file(cache_file)?;
        }
        Ok(())
    }

    /// Clear all cached data
    pub fn clear_all(&self) -> CacheResult<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == FILE_EXTENSION) {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the total size of cached data
    pub fn total_size(&self) -> CacheResult<u64> {
        let mut total = 0u64;
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    if let Ok(meta) = entry.metadata() {
                        total += meta.len();
                    }
                }
            }
        }
        Ok(total)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize cache manager")
    }
}

// Convenience functions for simpler API
/// Save an entry to cache using default settings
pub fn save(entry: &Entry) -> CacheResult<()> {
    CacheManager::new()?.save(entry)
}

/// Load an entry from cache using default settings
pub fn load(path: &Path) -> CacheResult<Entry> {
    CacheManager::new()?.load(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_roundtrip() {
        let cache_dir = tempdir().unwrap();
        let manager = CacheManager {
            cache_dir: cache_dir.path().to_path_buf(),
            max_age: Duration::from_secs(3600),
            use_compression: true,
        };

        let entry = Entry {
            name: "test".to_string(),
            size: 1000,
            path: PathBuf::from("/test/path"),
            is_dir: true,
            children: vec![],
            modified: None,
            file_count: 5,
            dir_count: 2,
            ..Default::default()
        };

        manager.save(&entry).unwrap();
        let loaded = manager.load(&entry.path).unwrap();

        assert_eq!(loaded.name, entry.name);
        assert_eq!(loaded.size, entry.size);
        assert_eq!(loaded.file_count, entry.file_count);
    }

    #[test]
    fn test_cache_expiry() {
        let cache_dir = tempdir().unwrap();
        let manager = CacheManager {
            cache_dir: cache_dir.path().to_path_buf(),
            max_age: Duration::from_secs(0), // Immediate expiry
            use_compression: true,
        };

        let entry = Entry {
            path: PathBuf::from("/test/path"),
            ..Default::default()
        };

        manager.save(&entry).unwrap();

        // Should be expired immediately
        std::thread::sleep(Duration::from_millis(10));
        assert!(matches!(
            manager.load(&entry.path),
            Err(CacheError::Expired(_))
        ));
    }
}
