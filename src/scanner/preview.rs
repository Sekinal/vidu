//! File preview functionality

use crate::constants::preview::{MAX_BYTES, MAX_LINES};
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;

/// Preview result
#[derive(Debug)]
pub struct FilePreview {
    pub lines: Vec<String>,
    pub truncated: bool,
    pub is_binary: bool,
    pub total_lines: Option<usize>,
}

impl FilePreview {
    /// Create an empty preview
    pub fn empty() -> Self {
        Self {
            lines: vec!["(empty file)".to_string()],
            truncated: false,
            is_binary: false,
            total_lines: Some(0),
        }
    }

    /// Create a binary file preview
    pub fn binary() -> Self {
        Self {
            lines: vec![
                "Binary file - cannot preview".to_string(),
                String::new(),
                "Press 'p' to close preview".to_string(),
            ],
            truncated: false,
            is_binary: true,
            total_lines: None,
        }
    }

    /// Create an error preview
    pub fn error(msg: &str) -> Self {
        Self {
            lines: vec![
                format!("Cannot read file: {}", msg),
                String::new(),
                "Press 'p' to close preview".to_string(),
            ],
            truncated: false,
            is_binary: false,
            total_lines: None,
        }
    }

    /// Create a "file too large" preview
    pub fn too_large(size: u64) -> Self {
        Self {
            lines: vec![
                format!("File too large to preview ({} bytes)", size),
                String::new(),
                "Press 'p' to close preview".to_string(),
            ],
            truncated: false,
            is_binary: false,
            total_lines: None,
        }
    }
}

/// Read a preview of file contents
pub fn read_file_preview(path: &Path) -> Result<FilePreview> {
    read_file_preview_with_limits(path, MAX_LINES, MAX_BYTES)
}

/// Read a preview with custom limits
pub fn read_file_preview_with_limits(
    path: &Path,
    max_lines: usize,
    max_bytes: usize,
) -> Result<FilePreview> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    // Don't try to preview very large files
    if metadata.len() > max_bytes as u64 {
        return Ok(FilePreview::too_large(metadata.len()));
    }

    let reader = BufReader::new(file);
    let mut lines = Vec::with_capacity(max_lines.min(1000));
    let mut truncated = false;

    for (i, line_result) in reader.lines().enumerate() {
        if i >= max_lines {
            truncated = true;
            break;
        }

        match line_result {
            Ok(line) => {
                // Check for binary content
                if line
                    .chars()
                    .any(|c| c.is_control() && c != '\t' && c != '\n')
                {
                    return Ok(FilePreview::binary());
                }
                lines.push(line);
            }
            Err(_) => {
                return Ok(FilePreview::binary());
            }
        }
    }

    if lines.is_empty() {
        return Ok(FilePreview::empty());
    }

    Ok(FilePreview {
        total_lines: if truncated { None } else { Some(lines.len()) },
        lines,
        truncated,
        is_binary: false,
    })
}

/// Create a directory info preview
pub fn directory_preview(
    name: &str,
    size: u64,
    file_count: usize,
    dir_count: usize,
    modified: &str,
) -> Vec<String> {
    vec![
        format!("Directory: {}", name),
        String::new(),
        format!("Total size: {}", crate::utils::format_bytes(size)),
        format!("Files: {}", file_count),
        format!("Directories: {}", dir_count),
        format!("Modified: {}", modified),
        String::new(),
        "Press Enter to open this directory".to_string(),
        "Press 'p' or Esc to close preview".to_string(),
    ]
}

/// Create a non-previewable file info
pub fn file_info_preview(name: &str, size: u64, modified: &str) -> Vec<String> {
    vec![
        format!("File: {}", name),
        String::new(),
        format!("Size: {}", crate::utils::format_bytes(size)),
        format!("Modified: {}", modified),
        String::new(),
        "Binary or unknown file type - cannot preview".to_string(),
        String::new(),
        "Press 'p' to close preview".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_preview_text_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello, World!").unwrap();
        writeln!(file, "This is a test.").unwrap();

        let preview = read_file_preview(file.path()).unwrap();
        assert!(!preview.is_binary);
        assert!(!preview.truncated);
        assert_eq!(preview.lines.len(), 2);
    }

    #[test]
    fn test_preview_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let preview = read_file_preview(file.path()).unwrap();
        assert!(!preview.is_binary);
        assert_eq!(preview.lines[0], "(empty file)");
    }
}
