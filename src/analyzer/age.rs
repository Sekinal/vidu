//! Age-based file analysis
//!
//! Identifies old and unused files based on modification and access times

use crate::scanner::Entry;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Age bucket for grouping files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AgeBucket {
    /// Less than 24 hours
    Today,
    /// 1-7 days
    ThisWeek,
    /// 7-30 days
    ThisMonth,
    /// 30-90 days
    LastQuarter,
    /// 90-365 days
    ThisYear,
    /// More than 1 year
    Older,
    /// Unknown age
    Unknown,
}

impl AgeBucket {
    /// Get the bucket for a given age in days
    pub fn from_days(days: u64) -> Self {
        match days {
            0 => AgeBucket::Today,
            1..=7 => AgeBucket::ThisWeek,
            8..=30 => AgeBucket::ThisMonth,
            31..=90 => AgeBucket::LastQuarter,
            91..=365 => AgeBucket::ThisYear,
            _ => AgeBucket::Older,
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            AgeBucket::Today => "Today",
            AgeBucket::ThisWeek => "This Week",
            AgeBucket::ThisMonth => "This Month",
            AgeBucket::LastQuarter => "Last 3 Months",
            AgeBucket::ThisYear => "This Year",
            AgeBucket::Older => "Older than 1 Year",
            AgeBucket::Unknown => "Unknown",
        }
    }

    /// Get a short label for compact display
    pub fn short_label(&self) -> &'static str {
        match self {
            AgeBucket::Today => "<1d",
            AgeBucket::ThisWeek => "<1w",
            AgeBucket::ThisMonth => "<1m",
            AgeBucket::LastQuarter => "<3m",
            AgeBucket::ThisYear => "<1y",
            AgeBucket::Older => ">1y",
            AgeBucket::Unknown => "?",
        }
    }
}

/// Statistics for an age bucket
#[derive(Debug, Clone, Default)]
pub struct AgeBucketStats {
    /// Number of files
    pub count: usize,
    /// Total size in bytes
    pub size: u64,
    /// Oldest file info (path, age in days)
    pub oldest: Option<(PathBuf, u64)>,
}

impl AgeBucketStats {
    /// Add a file to these stats
    pub fn add(&mut self, path: &PathBuf, size: u64, age_days: u64) {
        self.count += 1;
        self.size += size;

        match &self.oldest {
            None => self.oldest = Some((path.clone(), age_days)),
            Some((_, old_age)) if age_days > *old_age => {
                self.oldest = Some((path.clone(), age_days));
            }
            _ => {}
        }
    }
}

/// An old file candidate for cleanup
#[derive(Debug, Clone)]
pub struct OldFile {
    /// Path to the file
    pub path: PathBuf,
    /// File size
    pub size: u64,
    /// Age in days (modification)
    pub mod_age_days: u64,
    /// Access age in days (if available)
    pub access_age_days: Option<u64>,
}

/// Complete age analysis result
#[derive(Debug, Clone)]
pub struct AgeAnalysis {
    /// Stats by modification age bucket
    pub by_mod_age: std::collections::HashMap<AgeBucket, AgeBucketStats>,
    /// Stats by access age bucket (if available)
    pub by_access_age: std::collections::HashMap<AgeBucket, AgeBucketStats>,
    /// Old files (older than threshold)
    pub old_files: Vec<OldFile>,
    /// Threshold in days for "old" files
    pub old_threshold_days: u64,
    /// Total files analyzed
    pub total_files: usize,
    /// Total size analyzed
    pub total_size: u64,
}

impl AgeAnalysis {
    /// Create new analysis with default threshold (365 days)
    pub fn new() -> Self {
        Self::with_threshold(365)
    }

    /// Create new analysis with custom threshold
    pub fn with_threshold(days: u64) -> Self {
        Self {
            by_mod_age: std::collections::HashMap::new(),
            by_access_age: std::collections::HashMap::new(),
            old_files: Vec::new(),
            old_threshold_days: days,
            total_files: 0,
            total_size: 0,
        }
    }

    /// Analyze an entry tree
    pub fn from_entry(entry: &Entry) -> Self {
        Self::from_entry_with_threshold(entry, 365)
    }

    /// Analyze with custom threshold
    pub fn from_entry_with_threshold(entry: &Entry, threshold_days: u64) -> Self {
        let mut analysis = Self::with_threshold(threshold_days);
        let now = SystemTime::now();
        analysis.collect(entry, &now);
        // Sort old files by size (largest first)
        analysis.old_files.sort_by(|a, b| b.size.cmp(&a.size));
        analysis
    }

    /// Recursively collect stats
    fn collect(&mut self, entry: &Entry, now: &SystemTime) {
        if entry.is_dir {
            for child in &entry.children {
                self.collect(child, now);
            }
        } else {
            self.total_files += 1;
            self.total_size += entry.size;

            // Modification age
            let mod_age_days = entry
                .modified
                .and_then(|m| now.duration_since(m).ok())
                .map(|d| d.as_secs() / 86400);

            let mod_bucket = mod_age_days
                .map(AgeBucket::from_days)
                .unwrap_or(AgeBucket::Unknown);

            self.by_mod_age
                .entry(mod_bucket)
                .or_default()
                .add(&entry.path, entry.size, mod_age_days.unwrap_or(0));

            // Access age (if available)
            let access_age_days = entry
                .accessed
                .and_then(|a| now.duration_since(a).ok())
                .map(|d| d.as_secs() / 86400);

            if let Some(access_days) = access_age_days {
                let access_bucket = AgeBucket::from_days(access_days);
                self.by_access_age
                    .entry(access_bucket)
                    .or_default()
                    .add(&entry.path, entry.size, access_days);
            }

            // Track old files
            if let Some(days) = mod_age_days {
                if days >= self.old_threshold_days {
                    self.old_files.push(OldFile {
                        path: entry.path.clone(),
                        size: entry.size,
                        mod_age_days: days,
                        access_age_days,
                    });
                }
            }
        }
    }

    /// Get buckets sorted by size (largest first)
    pub fn mod_buckets_by_size(&self) -> Vec<(AgeBucket, &AgeBucketStats)> {
        let mut buckets: Vec<_> = self.by_mod_age.iter().map(|(k, v)| (*k, v)).collect();
        buckets.sort_by(|a, b| b.1.size.cmp(&a.1.size));
        buckets
    }

    /// Get buckets in chronological order
    pub fn mod_buckets_chronological(&self) -> Vec<(AgeBucket, &AgeBucketStats)> {
        let mut buckets: Vec<_> = self.by_mod_age.iter().map(|(k, v)| (*k, v)).collect();
        buckets.sort_by(|a, b| a.0.cmp(&b.0));
        buckets
    }

    /// Get total size of files older than threshold
    pub fn old_files_size(&self) -> u64 {
        self.old_files.iter().map(|f| f.size).sum()
    }

    /// Get top N old files by size
    pub fn top_old_files(&self, n: usize) -> &[OldFile] {
        let end = n.min(self.old_files.len());
        &self.old_files[..end]
    }

    /// Get percentage of space used by old files
    pub fn old_files_percentage(&self) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }
        (self.old_files_size() as f64 / self.total_size as f64) * 100.0
    }
}

impl Default for AgeAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate age in days from a SystemTime
pub fn age_in_days(time: SystemTime) -> Option<u64> {
    SystemTime::now()
        .duration_since(time)
        .ok()
        .map(|d| d.as_secs() / 86400)
}

/// Format age in human readable form
pub fn format_age(days: u64) -> String {
    if days == 0 {
        "today".to_string()
    } else if days == 1 {
        "yesterday".to_string()
    } else if days < 7 {
        format!("{} days ago", days)
    } else if days < 30 {
        let weeks = days / 7;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if days < 365 {
        let months = days / 30;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = days / 365;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(name: &str, size: u64, age_days: u64) -> Entry {
        let modified = SystemTime::now() - Duration::from_secs(age_days * 86400);
        Entry {
            name: name.to_string(),
            path: PathBuf::from(name),
            size,
            modified: Some(modified),
            ..Default::default()
        }
    }

    #[test]
    fn test_age_bucket_from_days() {
        assert_eq!(AgeBucket::from_days(0), AgeBucket::Today);
        assert_eq!(AgeBucket::from_days(5), AgeBucket::ThisWeek);
        assert_eq!(AgeBucket::from_days(20), AgeBucket::ThisMonth);
        assert_eq!(AgeBucket::from_days(60), AgeBucket::LastQuarter);
        assert_eq!(AgeBucket::from_days(200), AgeBucket::ThisYear);
        assert_eq!(AgeBucket::from_days(500), AgeBucket::Older);
    }

    #[test]
    fn test_age_analysis() {
        let root = Entry {
            name: "root".to_string(),
            path: PathBuf::from("root"),
            is_dir: true,
            children: vec![
                create_test_entry("new.txt", 100, 1),
                create_test_entry("old.txt", 500, 400),
            ],
            modified: Some(SystemTime::now()),
            ..Default::default()
        };

        let analysis = AgeAnalysis::from_entry(&root);

        assert_eq!(analysis.total_files, 2);
        assert_eq!(analysis.old_files.len(), 1);
        assert!(analysis.old_files[0].path.to_string_lossy().contains("old"));
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(0), "today");
        assert_eq!(format_age(1), "yesterday");
        assert_eq!(format_age(5), "5 days ago");
        assert_eq!(format_age(14), "2 weeks ago");
        assert_eq!(format_age(60), "2 months ago");
        assert_eq!(format_age(730), "2 years ago");
    }
}
