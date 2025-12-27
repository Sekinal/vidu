//! Package manager and browser cache detection
//!
//! Detects common cache locations for:
//! - Package managers: npm, cargo, pip, maven, gradle
//! - Browsers: Chrome, Firefox, Edge, Brave

use std::path::{Path, PathBuf};

/// A detected cache location
#[derive(Debug, Clone)]
pub struct CacheLocation {
    /// Name of the cache
    pub name: &'static str,
    /// Path to the cache
    pub path: PathBuf,
    /// Description
    pub description: &'static str,
    /// Category
    pub category: CacheCategory,
    /// Size in bytes (if known)
    pub size: Option<u64>,
}

/// Category of cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCategory {
    PackageManager,
    Browser,
    BuildTool,
    System,
}

impl CacheCategory {
    pub fn label(&self) -> &'static str {
        match self {
            CacheCategory::PackageManager => "Package Manager",
            CacheCategory::Browser => "Browser",
            CacheCategory::BuildTool => "Build Tool",
            CacheCategory::System => "System",
        }
    }
}

/// Cache detector
pub struct CacheDetector {
    home_dir: PathBuf,
}

impl CacheDetector {
    pub fn new() -> Option<Self> {
        dirs::home_dir().map(|home_dir| Self { home_dir })
    }

    /// Detect all known cache locations
    pub fn detect_all(&self) -> Vec<CacheLocation> {
        let mut caches = Vec::new();

        // Package manager caches
        caches.extend(self.detect_npm_cache());
        caches.extend(self.detect_cargo_cache());
        caches.extend(self.detect_pip_cache());
        caches.extend(self.detect_maven_cache());
        caches.extend(self.detect_gradle_cache());
        caches.extend(self.detect_go_cache());
        caches.extend(self.detect_composer_cache());

        // Browser caches
        caches.extend(self.detect_chrome_cache());
        caches.extend(self.detect_firefox_cache());
        caches.extend(self.detect_brave_cache());

        // System caches
        caches.extend(self.detect_system_cache());

        // Filter to only existing paths and calculate sizes
        caches
            .into_iter()
            .filter(|c| c.path.exists())
            .map(|mut c| {
                c.size = Some(dir_size(&c.path));
                c
            })
            .collect()
    }

    /// Detect by category
    pub fn detect_by_category(&self, category: CacheCategory) -> Vec<CacheLocation> {
        self.detect_all()
            .into_iter()
            .filter(|c| c.category == category)
            .collect()
    }

    // Package manager detection methods

    fn detect_npm_cache(&self) -> Vec<CacheLocation> {
        vec![
            CacheLocation {
                name: "npm cache",
                path: self.home_dir.join(".npm/_cacache"),
                description: "npm package cache",
                category: CacheCategory::PackageManager,
                size: None,
            },
            CacheLocation {
                name: "npm logs",
                path: self.home_dir.join(".npm/_logs"),
                description: "npm log files",
                category: CacheCategory::PackageManager,
                size: None,
            },
            CacheLocation {
                name: "yarn cache",
                path: self.home_dir.join(".yarn/cache"),
                description: "Yarn package cache",
                category: CacheCategory::PackageManager,
                size: None,
            },
            CacheLocation {
                name: "pnpm store",
                path: self.home_dir.join(".local/share/pnpm/store"),
                description: "pnpm content-addressable store",
                category: CacheCategory::PackageManager,
                size: None,
            },
        ]
    }

    fn detect_cargo_cache(&self) -> Vec<CacheLocation> {
        vec![
            CacheLocation {
                name: "Cargo registry cache",
                path: self.home_dir.join(".cargo/registry/cache"),
                description: "Cargo crate cache",
                category: CacheCategory::PackageManager,
                size: None,
            },
            CacheLocation {
                name: "Cargo git checkouts",
                path: self.home_dir.join(".cargo/git/checkouts"),
                description: "Cargo git dependencies",
                category: CacheCategory::PackageManager,
                size: None,
            },
        ]
    }

    fn detect_pip_cache(&self) -> Vec<CacheLocation> {
        vec![
            CacheLocation {
                name: "pip cache",
                path: self.home_dir.join(".cache/pip"),
                description: "Python pip cache",
                category: CacheCategory::PackageManager,
                size: None,
            },
            CacheLocation {
                name: "pipx cache",
                path: self.home_dir.join(".local/pipx/.cache"),
                description: "pipx cache",
                category: CacheCategory::PackageManager,
                size: None,
            },
        ]
    }

    fn detect_maven_cache(&self) -> Vec<CacheLocation> {
        vec![CacheLocation {
            name: "Maven repository",
            path: self.home_dir.join(".m2/repository"),
            description: "Maven local repository",
            category: CacheCategory::PackageManager,
            size: None,
        }]
    }

    fn detect_gradle_cache(&self) -> Vec<CacheLocation> {
        vec![
            CacheLocation {
                name: "Gradle cache",
                path: self.home_dir.join(".gradle/caches"),
                description: "Gradle build cache",
                category: CacheCategory::BuildTool,
                size: None,
            },
            CacheLocation {
                name: "Gradle wrapper",
                path: self.home_dir.join(".gradle/wrapper/dists"),
                description: "Gradle wrapper distributions",
                category: CacheCategory::BuildTool,
                size: None,
            },
        ]
    }

    fn detect_go_cache(&self) -> Vec<CacheLocation> {
        vec![CacheLocation {
            name: "Go module cache",
            path: self.home_dir.join("go/pkg/mod/cache"),
            description: "Go module cache",
            category: CacheCategory::PackageManager,
            size: None,
        }]
    }

    fn detect_composer_cache(&self) -> Vec<CacheLocation> {
        vec![CacheLocation {
            name: "Composer cache",
            path: self.home_dir.join(".composer/cache"),
            description: "PHP Composer cache",
            category: CacheCategory::PackageManager,
            size: None,
        }]
    }

    // Browser cache detection

    fn detect_chrome_cache(&self) -> Vec<CacheLocation> {
        let mut caches = Vec::new();

        // Linux
        let linux_path = self.home_dir.join(".cache/google-chrome");
        if linux_path.exists() {
            caches.push(CacheLocation {
                name: "Chrome cache",
                path: linux_path.join("Default/Cache"),
                description: "Google Chrome browser cache",
                category: CacheCategory::Browser,
                size: None,
            });
            caches.push(CacheLocation {
                name: "Chrome Code Cache",
                path: linux_path.join("Default/Code Cache"),
                description: "Chrome JavaScript cache",
                category: CacheCategory::Browser,
                size: None,
            });
        }

        // macOS
        let mac_path = self
            .home_dir
            .join("Library/Caches/Google/Chrome/Default/Cache");
        if mac_path.exists() {
            caches.push(CacheLocation {
                name: "Chrome cache",
                path: mac_path,
                description: "Google Chrome browser cache",
                category: CacheCategory::Browser,
                size: None,
            });
        }

        caches
    }

    fn detect_firefox_cache(&self) -> Vec<CacheLocation> {
        let mut caches = Vec::new();

        // Linux
        let linux_path = self.home_dir.join(".cache/mozilla/firefox");
        if linux_path.exists() {
            caches.push(CacheLocation {
                name: "Firefox cache",
                path: linux_path,
                description: "Firefox browser cache",
                category: CacheCategory::Browser,
                size: None,
            });
        }

        // macOS
        let mac_path = self.home_dir.join("Library/Caches/Firefox/Profiles");
        if mac_path.exists() {
            caches.push(CacheLocation {
                name: "Firefox cache",
                path: mac_path,
                description: "Firefox browser cache",
                category: CacheCategory::Browser,
                size: None,
            });
        }

        caches
    }

    fn detect_brave_cache(&self) -> Vec<CacheLocation> {
        let mut caches = Vec::new();

        // Linux
        let linux_path = self.home_dir.join(".cache/BraveSoftware/Brave-Browser");
        if linux_path.exists() {
            caches.push(CacheLocation {
                name: "Brave cache",
                path: linux_path.join("Default/Cache"),
                description: "Brave browser cache",
                category: CacheCategory::Browser,
                size: None,
            });
        }

        caches
    }

    fn detect_system_cache(&self) -> Vec<CacheLocation> {
        vec![
            CacheLocation {
                name: "Thumbnails",
                path: self.home_dir.join(".cache/thumbnails"),
                description: "Image thumbnail cache",
                category: CacheCategory::System,
                size: None,
            },
            CacheLocation {
                name: "fontconfig",
                path: self.home_dir.join(".cache/fontconfig"),
                description: "Font configuration cache",
                category: CacheCategory::System,
                size: None,
            },
            CacheLocation {
                name: "mesa shader cache",
                path: self.home_dir.join(".cache/mesa_shader_cache"),
                description: "GPU shader cache",
                category: CacheCategory::System,
                size: None,
            },
        ]
    }
}

impl Default for CacheDetector {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            home_dir: PathBuf::from("/"),
        })
    }
}

/// Calculate directory size recursively
fn dir_size(path: &Path) -> u64 {
    let mut size = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                size += path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                size += dir_size(&path);
            }
        }
    }

    size
}

/// Get total cache size
pub fn total_cache_size() -> u64 {
    CacheDetector::new()
        .map(|d| d.detect_all().iter().filter_map(|c| c.size).sum())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_detector() {
        if let Some(detector) = CacheDetector::new() {
            let caches = detector.detect_all();
            // Just verify it runs without panicking
            // Actual caches depend on the system
            for cache in &caches {
                assert!(!cache.name.is_empty());
                assert!(cache.path.exists());
            }
        }
    }

    #[test]
    fn test_cache_categories() {
        assert_eq!(CacheCategory::PackageManager.label(), "Package Manager");
        assert_eq!(CacheCategory::Browser.label(), "Browser");
    }
}
