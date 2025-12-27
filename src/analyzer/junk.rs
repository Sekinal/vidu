//! Junk file and directory detection

use super::patterns::{JunkPatterns, BUILTIN_PATTERNS};
use crate::config::JunkConfig;
use crate::scanner::{Entry, JunkType};

/// Junk detector with configurable patterns
pub struct JunkDetector {
    patterns: JunkPatterns,
    protected: Vec<String>,
    min_age_days: u64,
}

impl JunkDetector {
    /// Create a new detector with built-in patterns
    pub fn new() -> Self {
        Self {
            patterns: JunkPatterns::with_builtin(),
            protected: vec![
                ".git".to_string(),
                ".ssh".to_string(),
                ".gnupg".to_string(),
                ".config".to_string(),
            ],
            min_age_days: 0,
        }
    }

    /// Create a detector from config
    pub fn from_config(config: &JunkConfig) -> Self {
        let mut patterns = JunkPatterns::with_builtin();

        // Add custom patterns from config
        patterns.add_custom_directories(&config.junk_directories);
        patterns.add_custom_files(&config.junk_files);
        patterns.add_custom_files(&config.custom_patterns);

        Self {
            patterns,
            protected: config.protected_paths.clone(),
            min_age_days: config.min_age_days,
        }
    }

    /// Check if a path is protected from junk detection
    pub fn is_protected(&self, name: &str) -> bool {
        self.protected.iter().any(|p| p == name)
    }

    /// Detect junk type for a directory name
    pub fn detect_directory(&self, name: &str) -> Option<JunkType> {
        if self.is_protected(name) {
            return None;
        }
        self.patterns.is_junk_directory(name)
    }

    /// Detect junk type for a file name
    pub fn detect_file(&self, name: &str) -> Option<JunkType> {
        if self.is_protected(name) {
            return None;
        }
        self.patterns.is_junk_file(name)
    }

    /// Detect junk type for an entry
    pub fn detect_entry(&self, entry: &Entry) -> Option<JunkType> {
        if self.is_protected(&entry.name) {
            return None;
        }

        // Check for old files if min_age_days is configured
        if self.min_age_days > 0 {
            if let Some(age) = entry.age_days() {
                if age >= self.min_age_days {
                    return Some(JunkType::OldFile);
                }
            }
        }

        if entry.is_dir {
            self.detect_directory(&entry.name)
        } else {
            self.detect_file(&entry.name)
        }
    }

    /// Recursively mark junk in an entry tree
    pub fn mark_junk(&self, entry: &mut Entry) {
        // Check this entry
        if let Some(junk_type) = self.detect_entry(entry) {
            entry.junk_type = Some(junk_type);
        }

        // Recursively check children
        for child in &mut entry.children {
            self.mark_junk(child);
        }
    }
}

impl Default for JunkDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick check if a directory name is junk (uses builtin patterns)
pub fn is_junk_directory(name: &str) -> bool {
    BUILTIN_PATTERNS.is_junk_directory(name).is_some()
}

/// Quick check if a file name is junk (uses builtin patterns)
pub fn is_junk_file(name: &str) -> bool {
    BUILTIN_PATTERNS.is_junk_file(name).is_some()
}

/// Detect junk type for a name
pub fn detect_junk_type(name: &str, is_dir: bool) -> Option<JunkType> {
    if is_dir {
        BUILTIN_PATTERNS.is_junk_directory(name)
    } else {
        BUILTIN_PATTERNS.is_junk_file(name)
    }
}

/// Detect junk in an entry (non-recursive)
pub fn detect_junk(entry: &Entry) -> Option<JunkType> {
    detect_junk_type(&entry.name, entry.is_dir)
}

/// Statistics about detected junk
#[derive(Debug, Default, Clone)]
pub struct JunkStats {
    /// Total junk entries
    pub count: usize,
    /// Total junk size in bytes
    pub size: u64,
    /// Count by type
    pub by_type: std::collections::HashMap<JunkType, (usize, u64)>,
}

impl JunkStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entry to stats
    pub fn add(&mut self, junk_type: JunkType, size: u64) {
        self.count += 1;
        self.size += size;

        let entry = self.by_type.entry(junk_type).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += size;
    }

    /// Collect stats from an entry tree
    pub fn from_entry(entry: &Entry) -> Self {
        let mut stats = Self::new();
        Self::collect_stats(&mut stats, entry);
        stats
    }

    fn collect_stats(stats: &mut Self, entry: &Entry) {
        if let Some(junk_type) = entry.junk_type {
            stats.add(junk_type, entry.size);
        }

        for child in &entry.children {
            Self::collect_stats(stats, child);
        }
    }

    /// Get top junk types by size
    pub fn top_by_size(&self, n: usize) -> Vec<(JunkType, usize, u64)> {
        let mut items: Vec<_> = self
            .by_type
            .iter()
            .map(|(&jt, &(count, size))| (jt, count, size))
            .collect();
        items.sort_by(|a, b| b.2.cmp(&a.2));
        items.truncate(n);
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_junk_detector() {
        let detector = JunkDetector::new();

        assert!(detector.detect_directory("node_modules").is_some());
        assert!(detector.detect_directory("target").is_some());
        assert!(detector.detect_directory("src").is_none());

        assert!(detector.detect_file(".DS_Store").is_some());
        assert!(detector.detect_file("file.tmp").is_some());
        assert!(detector.detect_file("main.rs").is_none());
    }

    #[test]
    fn test_protected_paths() {
        let detector = JunkDetector::new();

        // .git should be protected even though it might match patterns
        assert!(detector.is_protected(".git"));
        assert!(detector.detect_directory(".git").is_none());
    }

    #[test]
    fn test_quick_functions() {
        assert!(is_junk_directory("node_modules"));
        assert!(is_junk_directory("__pycache__"));
        assert!(!is_junk_directory("src"));

        assert!(is_junk_file(".DS_Store"));
        assert!(is_junk_file("test.tmp"));
        assert!(!is_junk_file("main.rs"));
    }

    #[test]
    fn test_junk_stats() {
        let mut stats = JunkStats::new();

        stats.add(JunkType::BuildArtifact, 1000);
        stats.add(JunkType::BuildArtifact, 2000);
        stats.add(JunkType::Cache, 500);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.size, 3500);

        let (count, size) = stats.by_type[&JunkType::BuildArtifact];
        assert_eq!(count, 2);
        assert_eq!(size, 3000);
    }

    #[test]
    fn test_detect_entry() {
        let entry = Entry {
            name: "node_modules".to_string(),
            is_dir: true,
            ..Default::default()
        };

        assert_eq!(detect_junk(&entry), Some(JunkType::BuildArtifact));

        let file_entry = Entry {
            name: ".DS_Store".to_string(),
            is_dir: false,
            ..Default::default()
        };

        assert_eq!(detect_junk(&file_entry), Some(JunkType::SystemJunk));
    }
}
