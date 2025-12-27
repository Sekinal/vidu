//! File type analysis
//!
//! Categorizes files by type and generates size breakdowns

use crate::scanner::{Entry, FileCategory};
use std::collections::HashMap;

/// Statistics for a file category
#[derive(Debug, Clone, Default)]
pub struct CategoryStats {
    /// Number of files
    pub count: usize,
    /// Total size in bytes
    pub size: u64,
    /// List of extensions in this category
    pub extensions: HashMap<String, (usize, u64)>,
}

impl CategoryStats {
    /// Add a file to these stats
    pub fn add(&mut self, size: u64, extension: Option<&str>) {
        self.count += 1;
        self.size += size;
        if let Some(ext) = extension {
            let entry = self.extensions.entry(ext.to_lowercase()).or_default();
            entry.0 += 1;
            entry.1 += size;
        }
    }

    /// Get top extensions by size
    pub fn top_extensions_by_size(&self, n: usize) -> Vec<(&str, usize, u64)> {
        let mut exts: Vec<_> = self
            .extensions
            .iter()
            .map(|(ext, (count, size))| (ext.as_str(), *count, *size))
            .collect();
        exts.sort_by(|a, b| b.2.cmp(&a.2));
        exts.truncate(n);
        exts
    }

    /// Get top extensions by count
    pub fn top_extensions_by_count(&self, n: usize) -> Vec<(&str, usize, u64)> {
        let mut exts: Vec<_> = self
            .extensions
            .iter()
            .map(|(ext, (count, size))| (ext.as_str(), *count, *size))
            .collect();
        exts.sort_by(|a, b| b.1.cmp(&a.1));
        exts.truncate(n);
        exts
    }
}

/// Complete file type analysis result
#[derive(Debug, Clone)]
pub struct FileTypeAnalysis {
    /// Stats by category
    pub by_category: HashMap<FileCategory, CategoryStats>,
    /// Total files analyzed
    pub total_files: usize,
    /// Total size analyzed
    pub total_size: u64,
}

impl FileTypeAnalysis {
    /// Create new empty analysis
    pub fn new() -> Self {
        Self {
            by_category: HashMap::new(),
            total_files: 0,
            total_size: 0,
        }
    }

    /// Analyze an entry tree
    pub fn from_entry(entry: &Entry) -> Self {
        let mut analysis = Self::new();
        analysis.collect(entry);
        analysis
    }

    /// Recursively collect stats from entry tree
    fn collect(&mut self, entry: &Entry) {
        if entry.is_dir {
            for child in &entry.children {
                self.collect(child);
            }
        } else {
            self.total_files += 1;
            self.total_size += entry.size;

            let ext = entry.path.extension().and_then(|e| e.to_str());
            let category = entry.category;

            self.by_category
                .entry(category)
                .or_default()
                .add(entry.size, ext);
        }
    }

    /// Get categories sorted by size (largest first)
    pub fn categories_by_size(&self) -> Vec<(FileCategory, &CategoryStats)> {
        let mut cats: Vec<_> = self.by_category.iter().map(|(k, v)| (*k, v)).collect();
        cats.sort_by(|a, b| b.1.size.cmp(&a.1.size));
        cats
    }

    /// Get categories sorted by count (most files first)
    pub fn categories_by_count(&self) -> Vec<(FileCategory, &CategoryStats)> {
        let mut cats: Vec<_> = self.by_category.iter().map(|(k, v)| (*k, v)).collect();
        cats.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        cats
    }

    /// Get percentage of total size for a category
    pub fn size_percentage(&self, category: FileCategory) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }
        self.by_category
            .get(&category)
            .map(|s| (s.size as f64 / self.total_size as f64) * 100.0)
            .unwrap_or(0.0)
    }

    /// Get all unique extensions with their stats, sorted by size
    pub fn all_extensions_by_size(&self) -> Vec<(String, usize, u64)> {
        let mut all: HashMap<String, (usize, u64)> = HashMap::new();

        for stats in self.by_category.values() {
            for (ext, (count, size)) in &stats.extensions {
                let entry = all.entry(ext.clone()).or_default();
                entry.0 += count;
                entry.1 += size;
            }
        }

        let mut result: Vec<_> = all
            .into_iter()
            .map(|(ext, (count, size))| (ext, count, size))
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result
    }
}

impl Default for FileTypeAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_entry(name: &str, size: u64, is_dir: bool) -> Entry {
        Entry {
            name: name.to_string(),
            path: PathBuf::from(name),
            size,
            is_dir,
            category: FileCategory::from_extension(
                PathBuf::from(name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(""),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn test_file_type_analysis() {
        let mut root = create_test_entry("root", 0, true);
        root.children = vec![
            create_test_entry("file.txt", 1000, false),
            create_test_entry("image.png", 2000, false),
            create_test_entry("code.rs", 500, false),
        ];

        let analysis = FileTypeAnalysis::from_entry(&root);

        assert_eq!(analysis.total_files, 3);
        assert_eq!(analysis.total_size, 3500);
    }

    #[test]
    fn test_categories_by_size() {
        let mut root = create_test_entry("root", 0, true);
        root.children = vec![
            create_test_entry("big.png", 5000, false),
            create_test_entry("small.txt", 100, false),
        ];

        let analysis = FileTypeAnalysis::from_entry(&root);
        let sorted = analysis.categories_by_size();

        assert!(!sorted.is_empty());
        // Image should be first (larger)
        assert_eq!(sorted[0].0, FileCategory::Image);
    }

    #[test]
    fn test_extension_tracking() {
        let mut root = create_test_entry("root", 0, true);
        root.children = vec![
            create_test_entry("a.rs", 100, false),
            create_test_entry("b.rs", 200, false),
            create_test_entry("c.py", 150, false),
        ];

        let analysis = FileTypeAnalysis::from_entry(&root);
        let exts = analysis.all_extensions_by_size();

        // rs should have 2 files, 300 bytes total
        let rs = exts.iter().find(|(e, _, _)| e == "rs").unwrap();
        assert_eq!(rs.1, 2);
        assert_eq!(rs.2, 300);
    }
}
