//! Junk detection patterns

use crate::scanner::JunkType;
use glob::Pattern;

/// A pattern for detecting junk files/directories
#[derive(Debug, Clone)]
pub struct JunkPattern {
    /// The glob pattern
    pub pattern: String,
    /// Type of junk this pattern detects
    pub junk_type: JunkType,
    /// Whether this is a directory pattern
    pub is_directory: bool,
    /// Description of what this pattern matches
    pub description: &'static str,
}

impl JunkPattern {
    /// Create a new directory pattern
    pub fn directory(pattern: &str, junk_type: JunkType, description: &'static str) -> Self {
        Self {
            pattern: pattern.to_string(),
            junk_type,
            is_directory: true,
            description,
        }
    }

    /// Create a new file pattern
    pub fn file(pattern: &str, junk_type: JunkType, description: &'static str) -> Self {
        Self {
            pattern: pattern.to_string(),
            junk_type,
            is_directory: false,
            description,
        }
    }

    /// Check if a name matches this pattern
    pub fn matches(&self, name: &str) -> bool {
        if let Ok(pat) = Pattern::new(&self.pattern) {
            pat.matches(name)
        } else {
            // Fall back to exact match
            name == self.pattern
        }
    }
}

/// Collection of junk patterns
#[derive(Debug, Clone)]
pub struct JunkPatterns {
    /// Directory patterns
    pub directories: Vec<JunkPattern>,
    /// File patterns
    pub files: Vec<JunkPattern>,
}

impl JunkPatterns {
    /// Create a new empty pattern collection
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Create pattern collection with built-in patterns
    pub fn with_builtin() -> Self {
        Self {
            directories: builtin_directory_patterns(),
            files: builtin_file_patterns(),
        }
    }

    /// Add a directory pattern
    pub fn add_directory(&mut self, pattern: JunkPattern) {
        self.directories.push(pattern);
    }

    /// Add a file pattern
    pub fn add_file(&mut self, pattern: JunkPattern) {
        self.files.push(pattern);
    }

    /// Add custom directory patterns from config
    pub fn add_custom_directories(&mut self, patterns: &[String]) {
        for pattern in patterns {
            self.directories.push(JunkPattern::directory(
                pattern,
                JunkType::BuildArtifact, // Default type for custom
                "Custom pattern",
            ));
        }
    }

    /// Add custom file patterns from config
    pub fn add_custom_files(&mut self, patterns: &[String]) {
        for pattern in patterns {
            self.files.push(JunkPattern::file(
                pattern,
                JunkType::Temporary, // Default type for custom
                "Custom pattern",
            ));
        }
    }

    /// Check if a directory name matches any pattern
    pub fn is_junk_directory(&self, name: &str) -> Option<JunkType> {
        for pattern in &self.directories {
            if pattern.matches(name) {
                return Some(pattern.junk_type);
            }
        }
        None
    }

    /// Check if a file name matches any pattern
    pub fn is_junk_file(&self, name: &str) -> Option<JunkType> {
        for pattern in &self.files {
            if pattern.matches(name) {
                return Some(pattern.junk_type);
            }
        }
        None
    }
}

impl Default for JunkPatterns {
    fn default() -> Self {
        Self::with_builtin()
    }
}

/// Built-in junk patterns (lazy static)
pub static BUILTIN_PATTERNS: std::sync::LazyLock<JunkPatterns> =
    std::sync::LazyLock::new(JunkPatterns::with_builtin);

/// Get built-in directory patterns
fn builtin_directory_patterns() -> Vec<JunkPattern> {
    vec![
        // Node.js / JavaScript
        JunkPattern::directory("node_modules", JunkType::BuildArtifact, "Node.js dependencies"),
        JunkPattern::directory(".npm", JunkType::PackageCache, "npm cache"),
        JunkPattern::directory(".yarn", JunkType::PackageCache, "Yarn cache"),
        JunkPattern::directory(".pnpm-store", JunkType::PackageCache, "pnpm store"),
        JunkPattern::directory("bower_components", JunkType::BuildArtifact, "Bower dependencies"),
        JunkPattern::directory(".next", JunkType::BuildArtifact, "Next.js build output"),
        JunkPattern::directory(".nuxt", JunkType::BuildArtifact, "Nuxt.js build output"),
        JunkPattern::directory(".output", JunkType::BuildArtifact, "Build output"),
        JunkPattern::directory(".vite", JunkType::Cache, "Vite cache"),
        JunkPattern::directory(".parcel-cache", JunkType::Cache, "Parcel cache"),
        JunkPattern::directory(".turbo", JunkType::Cache, "Turborepo cache"),
        // Rust
        JunkPattern::directory("target", JunkType::BuildArtifact, "Rust build output"),
        // Python
        JunkPattern::directory("__pycache__", JunkType::BuildArtifact, "Python bytecode cache"),
        JunkPattern::directory(".pytest_cache", JunkType::Cache, "pytest cache"),
        JunkPattern::directory(".mypy_cache", JunkType::Cache, "mypy cache"),
        JunkPattern::directory(".ruff_cache", JunkType::Cache, "ruff cache"),
        JunkPattern::directory(".tox", JunkType::BuildArtifact, "tox environments"),
        JunkPattern::directory("venv", JunkType::BuildArtifact, "Python virtual environment"),
        JunkPattern::directory(".venv", JunkType::BuildArtifact, "Python virtual environment"),
        JunkPattern::directory("env", JunkType::BuildArtifact, "Python environment"),
        JunkPattern::directory(".eggs", JunkType::BuildArtifact, "Python eggs"),
        JunkPattern::directory("*.egg-info", JunkType::BuildArtifact, "Python egg info"),
        // Java / JVM
        JunkPattern::directory(".gradle", JunkType::Cache, "Gradle cache"),
        JunkPattern::directory(".m2", JunkType::PackageCache, "Maven repository"),
        JunkPattern::directory("build", JunkType::BuildArtifact, "Build output"),
        JunkPattern::directory("out", JunkType::BuildArtifact, "Build output"),
        // Go
        JunkPattern::directory("vendor", JunkType::BuildArtifact, "Go vendor"),
        // Generic
        JunkPattern::directory("dist", JunkType::BuildArtifact, "Distribution output"),
        JunkPattern::directory(".cache", JunkType::Cache, "Generic cache"),
        JunkPattern::directory("tmp", JunkType::Temporary, "Temporary files"),
        JunkPattern::directory("temp", JunkType::Temporary, "Temporary files"),
        JunkPattern::directory("logs", JunkType::LogFile, "Log files"),
        JunkPattern::directory("coverage", JunkType::BuildArtifact, "Test coverage"),
        JunkPattern::directory(".coverage", JunkType::BuildArtifact, "Coverage data"),
        // IDE
        JunkPattern::directory(".idea", JunkType::Cache, "IntelliJ IDEA"),
        JunkPattern::directory(".vscode", JunkType::Cache, "VS Code settings"),
        // OS
        JunkPattern::directory(".Trash", JunkType::SystemJunk, "macOS Trash"),
        JunkPattern::directory(".Trashes", JunkType::SystemJunk, "macOS Trashes"),
        JunkPattern::directory(".Spotlight-V100", JunkType::SystemJunk, "Spotlight index"),
        JunkPattern::directory(".fseventsd", JunkType::SystemJunk, "FSEvents data"),
    ]
}

/// Get built-in file patterns
fn builtin_file_patterns() -> Vec<JunkPattern> {
    vec![
        // Temporary files
        JunkPattern::file("*.tmp", JunkType::Temporary, "Temporary file"),
        JunkPattern::file("*.temp", JunkType::Temporary, "Temporary file"),
        JunkPattern::file("*.swp", JunkType::Temporary, "Vim swap file"),
        JunkPattern::file("*.swo", JunkType::Temporary, "Vim swap file"),
        JunkPattern::file("*~", JunkType::Backup, "Backup file"),
        JunkPattern::file("*.bak", JunkType::Backup, "Backup file"),
        JunkPattern::file("*.backup", JunkType::Backup, "Backup file"),
        JunkPattern::file("*.old", JunkType::Backup, "Old file"),
        JunkPattern::file("*.orig", JunkType::Backup, "Original file"),
        // Log files
        JunkPattern::file("*.log", JunkType::LogFile, "Log file"),
        JunkPattern::file("*.log.*", JunkType::LogFile, "Rotated log file"),
        // System junk
        JunkPattern::file(".DS_Store", JunkType::SystemJunk, "macOS folder metadata"),
        JunkPattern::file("Thumbs.db", JunkType::SystemJunk, "Windows thumbnail cache"),
        JunkPattern::file("thumbs.db", JunkType::SystemJunk, "Windows thumbnail cache"),
        JunkPattern::file("desktop.ini", JunkType::SystemJunk, "Windows folder settings"),
        JunkPattern::file("Desktop.ini", JunkType::SystemJunk, "Windows folder settings"),
        JunkPattern::file("ehthumbs.db", JunkType::SystemJunk, "Windows Media Center"),
        JunkPattern::file(".directory", JunkType::SystemJunk, "KDE folder settings"),
        // Python
        JunkPattern::file("*.pyc", JunkType::BuildArtifact, "Python bytecode"),
        JunkPattern::file("*.pyo", JunkType::BuildArtifact, "Python optimized bytecode"),
        JunkPattern::file("*.pyd", JunkType::BuildArtifact, "Python extension"),
        // Compiled
        JunkPattern::file("*.o", JunkType::BuildArtifact, "Object file"),
        JunkPattern::file("*.obj", JunkType::BuildArtifact, "Object file"),
        JunkPattern::file("*.so", JunkType::BuildArtifact, "Shared object"),
        JunkPattern::file("*.dll", JunkType::BuildArtifact, "Dynamic library"),
        JunkPattern::file("*.dylib", JunkType::BuildArtifact, "Dynamic library"),
        JunkPattern::file("*.a", JunkType::BuildArtifact, "Static library"),
        JunkPattern::file("*.lib", JunkType::BuildArtifact, "Static library"),
        // Core dumps
        JunkPattern::file("core", JunkType::Temporary, "Core dump"),
        JunkPattern::file("core.*", JunkType::Temporary, "Core dump"),
        // Lock files (sometimes safe to delete)
        JunkPattern::file("*.lock", JunkType::Temporary, "Lock file"),
        JunkPattern::file(".*.lock", JunkType::Temporary, "Lock file"),
        // Coverage
        JunkPattern::file("*.gcno", JunkType::BuildArtifact, "GCC coverage"),
        JunkPattern::file("*.gcda", JunkType::BuildArtifact, "GCC coverage data"),
        JunkPattern::file("*.gcov", JunkType::BuildArtifact, "GCC coverage output"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let pattern = JunkPattern::file("*.tmp", JunkType::Temporary, "Test");
        assert!(pattern.matches("file.tmp"));
        assert!(pattern.matches("another.tmp"));
        assert!(!pattern.matches("file.txt"));
    }

    #[test]
    fn test_directory_pattern() {
        let pattern = JunkPattern::directory("node_modules", JunkType::BuildArtifact, "Test");
        assert!(pattern.matches("node_modules"));
        assert!(!pattern.matches("node_modules_bak"));
    }

    #[test]
    fn test_builtin_patterns() {
        let patterns = JunkPatterns::with_builtin();

        assert!(patterns.is_junk_directory("node_modules").is_some());
        assert!(patterns.is_junk_directory("target").is_some());
        assert!(patterns.is_junk_directory("__pycache__").is_some());
        assert!(patterns.is_junk_directory("random_dir").is_none());

        assert!(patterns.is_junk_file(".DS_Store").is_some());
        assert!(patterns.is_junk_file("Thumbs.db").is_some());
        assert!(patterns.is_junk_file("file.tmp").is_some());
        assert!(patterns.is_junk_file("important.txt").is_none());
    }

    #[test]
    fn test_custom_patterns() {
        let mut patterns = JunkPatterns::new();
        patterns.add_custom_directories(&["my_build".to_string()]);
        patterns.add_custom_files(&["*.custom".to_string()]);

        assert!(patterns.is_junk_directory("my_build").is_some());
        assert!(patterns.is_junk_file("test.custom").is_some());
    }
}
