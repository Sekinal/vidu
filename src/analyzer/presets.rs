//! Cleaning presets
//!
//! Predefined and custom cleaning configurations

use crate::scanner::{Entry, JunkType};
use std::path::PathBuf;

/// A cleaning preset
#[derive(Debug, Clone)]
pub struct CleaningPreset {
    /// Preset name
    pub name: String,
    /// Description
    pub description: String,
    /// Glob patterns to match
    pub patterns: Vec<String>,
    /// Junk types to include
    pub junk_types: Vec<JunkType>,
    /// Minimum file age in days (0 = any age)
    pub min_age_days: u64,
    /// Whether this is a built-in preset
    pub builtin: bool,
}

impl CleaningPreset {
    /// Create a new custom preset
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            patterns: Vec::new(),
            junk_types: Vec::new(),
            min_age_days: 0,
            builtin: false,
        }
    }

    /// Add patterns to match
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns = patterns;
        self
    }

    /// Add junk types to include
    pub fn with_junk_types(mut self, types: Vec<JunkType>) -> Self {
        self.junk_types = types;
        self
    }

    /// Set minimum age
    pub fn with_min_age(mut self, days: u64) -> Self {
        self.min_age_days = days;
        self
    }

    /// Mark as built-in
    pub fn as_builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Check if a path matches any pattern
    pub fn matches_path(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.patterns {
            // Simple glob matching
            if pattern.contains("**") {
                // Recursive pattern - check if any component matches
                let base = pattern.replace("**/", "");
                if path_str.contains(&base) {
                    return true;
                }
            } else if pattern.starts_with('*') {
                // Extension pattern
                let ext = &pattern[1..];
                if path_str.ends_with(ext) {
                    return true;
                }
            } else {
                // Exact name match
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == pattern {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if an entry matches this preset
    pub fn matches(&self, entry: &Entry) -> bool {
        // Check junk type
        if !self.junk_types.is_empty() {
            if let Some(jt) = entry.junk_type {
                if self.junk_types.contains(&jt) {
                    return true;
                }
            }
        }

        // Check patterns
        if self.matches_path(&entry.path) {
            return true;
        }

        false
    }

    /// Collect all matching entries from a tree
    pub fn collect_matches<'a>(&self, entry: &'a Entry) -> Vec<&'a Entry> {
        let mut matches = Vec::new();
        self.collect_recursive(entry, &mut matches);
        matches
    }

    fn collect_recursive<'a>(&self, entry: &'a Entry, matches: &mut Vec<&'a Entry>) {
        if self.matches(entry) {
            matches.push(entry);
        } else if entry.is_dir {
            for child in &entry.children {
                self.collect_recursive(child, matches);
            }
        }
    }

    /// Calculate total size of matching entries
    pub fn calculate_size(&self, entry: &Entry) -> u64 {
        self.collect_matches(entry).iter().map(|e| e.size).sum()
    }
}

/// Built-in presets
pub struct BuiltinPresets;

impl BuiltinPresets {
    /// Developer cleanup - build artifacts and dependencies
    pub fn developer() -> CleaningPreset {
        CleaningPreset::new("Developer", "Build artifacts and package dependencies")
            .with_patterns(vec![
                "**/node_modules".to_string(),
                "**/target".to_string(),
                "**/__pycache__".to_string(),
                "**/dist".to_string(),
                "**/build".to_string(),
                "**/.next".to_string(),
                "**/.nuxt".to_string(),
                "**/vendor".to_string(),
                "**/.gradle".to_string(),
            ])
            .with_junk_types(vec![JunkType::BuildArtifact, JunkType::PackageCache])
            .as_builtin()
    }

    /// Cache cleanup - all cached data
    pub fn caches() -> CleaningPreset {
        CleaningPreset::new("Caches", "Package manager and application caches")
            .with_patterns(vec![
                "**/.cache".to_string(),
                "**/.npm".to_string(),
                "**/Cache".to_string(),
                "**/cache".to_string(),
            ])
            .with_junk_types(vec![JunkType::Cache, JunkType::PackageCache])
            .as_builtin()
    }

    /// Temporary files cleanup
    pub fn temporary() -> CleaningPreset {
        CleaningPreset::new("Temporary", "Temporary and swap files")
            .with_patterns(vec![
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ])
            .with_junk_types(vec![JunkType::Temporary, JunkType::SystemJunk])
            .as_builtin()
    }

    /// Log files cleanup
    pub fn logs() -> CleaningPreset {
        CleaningPreset::new("Logs", "Log files and debug output")
            .with_patterns(vec![
                "*.log".to_string(),
                "**/logs".to_string(),
                "**/*.log.*".to_string(),
                "**/debug.log".to_string(),
                "**/error.log".to_string(),
            ])
            .with_junk_types(vec![JunkType::LogFile])
            .as_builtin()
    }

    /// Old files cleanup
    pub fn old_files(min_age_days: u64) -> CleaningPreset {
        CleaningPreset::new(
            "Old Files",
            format!("Files not modified in {} days", min_age_days),
        )
        .with_min_age(min_age_days)
        .with_junk_types(vec![JunkType::OldFile])
        .as_builtin()
    }

    /// Backup files cleanup
    pub fn backups() -> CleaningPreset {
        CleaningPreset::new("Backups", "Backup and auto-save files")
            .with_patterns(vec![
                "*.bak".to_string(),
                "*.backup".to_string(),
                "*.orig".to_string(),
                "*.old".to_string(),
                "*~".to_string(),
            ])
            .with_junk_types(vec![JunkType::Backup])
            .as_builtin()
    }

    /// Aggressive cleanup - everything
    pub fn aggressive() -> CleaningPreset {
        CleaningPreset::new("Aggressive", "All detected junk (use with caution)")
            .with_junk_types(vec![
                JunkType::BuildArtifact,
                JunkType::Cache,
                JunkType::Temporary,
                JunkType::LogFile,
                JunkType::Backup,
                JunkType::SystemJunk,
                JunkType::PackageCache,
            ])
            .as_builtin()
    }

    /// Get all built-in presets
    pub fn all() -> Vec<CleaningPreset> {
        vec![
            Self::developer(),
            Self::caches(),
            Self::temporary(),
            Self::logs(),
            Self::backups(),
            Self::old_files(365),
            Self::aggressive(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_entry(path: &str, junk_type: Option<JunkType>) -> Entry {
        Entry {
            name: PathBuf::from(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            path: PathBuf::from(path),
            size: 100,
            junk_type,
            modified: Some(SystemTime::now()),
            ..Default::default()
        }
    }

    #[test]
    fn test_pattern_matching() {
        let preset = CleaningPreset::new("Test", "Test preset")
            .with_patterns(vec!["*.log".to_string(), "node_modules".to_string()]);

        assert!(preset.matches_path(&PathBuf::from("/tmp/error.log")));
        assert!(preset.matches_path(&PathBuf::from("/project/node_modules")));
        assert!(!preset.matches_path(&PathBuf::from("/project/src/main.rs")));
    }

    #[test]
    fn test_junk_type_matching() {
        let preset =
            CleaningPreset::new("Test", "Test").with_junk_types(vec![JunkType::BuildArtifact]);

        let entry = create_test_entry("target", Some(JunkType::BuildArtifact));
        assert!(preset.matches(&entry));

        let entry2 = create_test_entry("src", None);
        assert!(!preset.matches(&entry2));
    }

    #[test]
    fn test_builtin_presets() {
        let presets = BuiltinPresets::all();
        assert!(!presets.is_empty());
        assert!(presets.iter().all(|p| p.builtin));
    }

    #[test]
    fn test_developer_preset() {
        let preset = BuiltinPresets::developer();

        assert!(preset.matches_path(&PathBuf::from("/project/node_modules")));
        assert!(preset.matches_path(&PathBuf::from("/rust/target")));
        assert!(preset.matches_path(&PathBuf::from("/python/__pycache__")));
    }
}
