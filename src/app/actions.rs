//! Application action handlers

use super::state::{App, AppState, SortMode, SortOrder};
use crate::scanner::{
    directory_preview, file_info_preview, read_file_preview, refresh_children, scan_with_progress,
    ScanOptions,
};
use crate::utils::{format_bytes, format_time, is_text_file};
use std::path::PathBuf;

impl App {
    // ==========================================
    // NAVIGATION
    // ==========================================

    pub fn move_up(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        if selected > 0 {
            self.table_state.select(Some(selected - 1));
        }
    }

    pub fn move_down(&mut self) {
        let visible_count = self.visible_children_count();
        let selected = self.table_state.selected().unwrap_or(0);
        if selected < visible_count.saturating_sub(1) {
            self.table_state.select(Some(selected + 1));
        }
    }

    pub fn go_to_top(&mut self) {
        if self.visible_children_count() > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub fn go_to_bottom(&mut self) {
        let len = self.visible_children_count();
        if len > 0 {
            self.table_state.select(Some(len - 1));
        }
    }

    pub fn page_up(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        let jump = self.visible_rows();
        self.table_state.select(Some(selected.saturating_sub(jump)));
    }

    pub fn page_down(&mut self) {
        let visible_count = self.visible_children_count();
        let selected = self.table_state.selected().unwrap_or(0);
        let jump = self.visible_rows();
        let new_pos = (selected + jump).min(visible_count.saturating_sub(1));
        self.table_state.select(Some(new_pos));
    }

    pub fn enter_dir(&mut self) {
        let Some(visible_idx) = self.table_state.selected() else {
            return;
        };

        // Map visible index to actual index
        let Some(actual_idx) = self.visible_to_actual_index(visible_idx) else {
            return;
        };

        let current = self.current_view();
        let child = &current.children[actual_idx];
        if !child.is_dir {
            // If it's a file, try to preview it
            self.toggle_preview();
            return;
        }

        // Check for errors
        if child.has_error() {
            self.error_msg = child.error.clone();
            return;
        }

        // Push actual index to nav_stack (not visible index)
        self.nav_stack.push(actual_idx);
        self.table_state.select(Some(0));
        self.apply_sort();
    }

    pub fn go_back(&mut self) {
        if let Some(actual_idx) = self.nav_stack.pop() {
            // Convert actual index back to visible index
            if let Some(visible_idx) = self.actual_to_visible_index(actual_idx) {
                self.table_state.select(Some(visible_idx));
            } else {
                // Item is hidden, select first visible item
                let visible_count = self.visible_children_count();
                if visible_count > 0 {
                    self.table_state.select(Some(0));
                } else {
                    self.table_state.select(None);
                }
            }
        }
    }

    pub fn go_to_root(&mut self) {
        self.nav_stack.clear();
        self.table_state.select(Some(0));
    }

    // ==========================================
    // SORTING
    // ==========================================

    pub fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.apply_sort();
        self.status_msg = format!("Sorted by {}", self.sort_mode.label());
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_order = self.sort_order.toggle();
        self.apply_sort();
        self.status_msg = format!("Sort order: {}", self.sort_order.label());
    }

    pub fn apply_sort(&mut self) {
        let sort_mode = self.sort_mode;
        let sort_order = self.sort_order;

        let current = self.current_view_mut();

        current.children.sort_by(|a, b| {
            let cmp = match sort_mode {
                SortMode::Size => a.size.cmp(&b.size),
                SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortMode::Modified => a.modified.cmp(&b.modified),
                SortMode::Count => a.file_count.cmp(&b.file_count),
            };

            match sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    // ==========================================
    // DELETION
    // ==========================================

    pub fn request_delete(&mut self) {
        if self.table_state.selected().is_some() {
            self.state = AppState::DeleteConfirm;
        }
    }

    pub fn confirm_delete(&mut self) {
        let Some(visible_idx) = self.table_state.selected() else {
            self.state = AppState::Browsing;
            return;
        };

        // Map visible index to actual index
        let Some(actual_idx) = self.visible_to_actual_index(visible_idx) else {
            self.state = AppState::Browsing;
            return;
        };

        // Get item info
        let (path, size) = {
            let view = self.current_view();
            let item = &view.children[actual_idx];
            (item.path.clone(), item.size)
        };

        // Use the same deletion progress system as batch cleaning
        self.pending_clean_items = vec![(path, size)];
        self.pending_clean_size = size;

        // Initialize deletion progress
        self.deletion_progress = Some(super::state::DeletionProgress {
            total_items: 1,
            completed_items: 0,
            total_bytes: size,
            freed_bytes: 0,
            current_path: String::new(),
            failed_items: 0,
            started_at: std::time::Instant::now(),
        });

        // Switch to deleting state
        self.state = AppState::Deleting;
    }

    // ==========================================
    // REFRESH & RESCAN
    // ==========================================

    pub fn refresh_current(&mut self) {
        self.status_msg = "Refreshing...".to_string();

        let options = ScanOptions::default().with_hidden(self.show_hidden);
        let current = self.current_view_mut();
        refresh_children(current, &options);

        self.apply_sort();

        // Update selection if out of bounds
        let len = self.current_view().children.len();
        if let Some(selected) = self.table_state.selected() {
            if selected >= len {
                self.table_state
                    .select(if len > 0 { Some(len - 1) } else { None });
            }
        }

        self.save_to_cache();
        self.status_msg = "Refreshed".to_string();
    }

    pub fn full_rescan(&mut self) {
        self.status_msg = "Rescanning...".to_string();

        let path = self.original_path.clone();
        let options = ScanOptions::default().with_hidden(self.show_hidden);
        self.root = scan_with_progress(path, &options, None);
        self.nav_stack.clear();

        if !self.root.children.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }

        self.save_to_cache();
        self.status_msg = format!(
            "Scanned {} files, {} dirs",
            self.root.file_count, self.root.dir_count
        );
    }

    // ==========================================
    // PREVIEW
    // ==========================================

    pub fn toggle_preview(&mut self) {
        if self.state == AppState::Preview {
            self.state = AppState::Browsing;
            return;
        }

        let Some(item) = self.selected_item() else {
            return;
        };

        if item.is_dir {
            self.preview_content = directory_preview(
                &item.name,
                item.size,
                item.file_count,
                item.dir_count,
                &format_time(item.modified),
            );
        } else if is_text_file(&item.name) {
            match read_file_preview(&item.path) {
                Ok(preview) => {
                    self.preview_content = preview.lines;
                }
                Err(e) => {
                    self.preview_content = vec![
                        format!("Cannot read file: {}", e),
                        String::new(),
                        "Press 'p' to close preview".to_string(),
                    ];
                }
            }
        } else {
            self.preview_content =
                file_info_preview(&item.name, item.size, &format_time(item.modified));
        }

        self.preview_scroll = 0;
        self.state = AppState::Preview;
    }

    // ==========================================
    // SEARCH
    // ==========================================

    pub fn execute_search(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }

        let results: Vec<usize> = {
            let current = self.current_view();
            current
                .children
                .iter()
                .enumerate()
                .filter(|(_, child)| child.name.to_lowercase().contains(&query))
                .map(|(idx, _)| idx)
                .collect()
        };

        self.search_results = results;

        if !self.search_results.is_empty() {
            self.search_index = 0;
            self.table_state.select(Some(self.search_results[0]));
            self.status_msg = format!(
                "Found {} matches - Use n/N to navigate",
                self.search_results.len()
            );
        } else {
            self.status_msg = format!("No matches for '{}'", self.search_query);
        }
    }

    // ==========================================
    // OTHER
    // ==========================================

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.status_msg = format!(
            "Hidden files: {}",
            if self.show_hidden { "shown" } else { "hidden" }
        );
        // Clamp selection to visible range after toggling
        let visible_count = self.visible_children_count();
        if let Some(selected) = self.table_state.selected() {
            if selected >= visible_count {
                if visible_count > 0 {
                    self.table_state.select(Some(visible_count - 1));
                } else {
                    self.table_state.select(None);
                }
            }
        }
    }

    pub fn toggle_mark(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };

        let path = item.path.clone();

        if self.marked_items.contains(&path) {
            self.marked_items.remove(&path);
        } else {
            self.marked_items.insert(path);
        }

        self.status_msg = format!("{} items marked", self.marked_items.len());

        // Move down after marking
        self.move_down();
    }

    // ==========================================
    // ANALYSIS NAVIGATION
    // ==========================================

    pub fn analysis_scroll_up(&mut self) {
        if self.analysis_selected > 0 {
            self.analysis_selected -= 1;
        }
    }

    pub fn analysis_scroll_down(&mut self, max_items: usize) {
        if self.analysis_selected < max_items.saturating_sub(1) {
            self.analysis_selected += 1;
        }
    }

    pub fn analysis_page_up(&mut self) {
        self.analysis_selected = self.analysis_selected.saturating_sub(10);
    }

    pub fn analysis_page_down(&mut self, max_items: usize) {
        self.analysis_selected = (self.analysis_selected + 10).min(max_items.saturating_sub(1));
    }

    pub fn analysis_go_to_top(&mut self) {
        self.analysis_selected = 0;
    }

    pub fn analysis_go_to_bottom(&mut self, max_items: usize) {
        self.analysis_selected = max_items.saturating_sub(1);
    }

    pub fn close_analysis(&mut self) {
        self.state = AppState::Browsing;
    }

    // ==========================================
    // CLEANING
    // ==========================================

    /// Prepare to clean marked items - shows confirmation
    pub fn prepare_clean_marked(&mut self) {
        if self.marked_items.is_empty() {
            self.status_msg = "No items marked for cleaning".to_string();
            return;
        }

        // Build items with sizes from marked paths
        let mut items: Vec<(PathBuf, u64)> = Vec::new();
        let mut total_size = 0u64;

        for path in &self.marked_items {
            // Try to find size from our tree first (fast)
            let size = self.find_entry_size(path).unwrap_or_else(|| {
                // Fall back to filesystem lookup (slower)
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.is_dir() {
                        self.get_dir_size(path)
                    } else {
                        metadata.len()
                    }
                } else {
                    0
                }
            });
            total_size += size;
            items.push((path.clone(), size));
        }

        self.pending_clean_items = items;
        self.pending_clean_size = total_size;
        self.state = AppState::CleaningConfirm;
    }

    /// Prepare to clean all junk items - shows confirmation
    pub fn prepare_clean_all_junk(&mut self) {
        let junk_entries = self.root.collect_junk();

        if junk_entries.is_empty() {
            self.status_msg = "No junk detected to clean".to_string();
            return;
        }

        // Build items with sizes from entries (already have sizes)
        let items: Vec<_> = junk_entries.iter().map(|e| (e.path.clone(), e.size)).collect();
        let total_size: u64 = junk_entries.iter().map(|e| e.size).sum();

        self.pending_clean_items = items;
        self.pending_clean_size = total_size;
        self.state = AppState::CleaningConfirm;
    }

    /// Execute the pending cleaning operation - starts deletion with progress
    pub fn execute_cleaning(&mut self) {
        if self.pending_clean_items.is_empty() {
            self.state = AppState::Browsing;
            return;
        }

        // Initialize deletion progress
        self.deletion_progress = Some(super::state::DeletionProgress {
            total_items: self.pending_clean_items.len(),
            completed_items: 0,
            total_bytes: self.pending_clean_size,
            freed_bytes: 0,
            current_path: String::new(),
            failed_items: 0,
            started_at: std::time::Instant::now(),
        });

        // Switch to deleting state - actual deletion happens in run_with_deleting
        self.state = AppState::Deleting;
    }

    /// Finish the deletion process and update state
    pub fn finish_deletion(&mut self, result: crate::cleaner::CleaningResult) {
        use std::io::Write;
        let start = std::time::Instant::now();
        let mut log = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open("/tmp/vidu_debug.log").ok();
        macro_rules! timing {
            ($($arg:tt)*) => {
                if let Some(ref mut f) = log {
                    let _ = writeln!(f, "[FINISH {:?}] {}", start.elapsed(), format!($($arg)*));
                    let _ = f.flush();
                }
            };
        }

        timing!("finish_deletion started");

        // Update disk available space
        self.disk_available = self.disk_available.saturating_add(result.bytes_freed);
        timing!("disk space updated");

        // Clear marked items that were successfully deleted
        for (path, _) in &self.pending_clean_items {
            if !path.exists() {
                self.marked_items.remove(path);
            }
        }
        timing!("marked items cleared");

        // Remove only the specific deleted entries from the tree (not checking every path)
        let deleted_paths: Vec<_> = self.pending_clean_items.iter().map(|(p, _)| p.clone()).collect();
        self.remove_specific_entries(&deleted_paths);
        timing!("deleted entries removed from tree");

        // Invalidate analysis caches
        self.invalidate_analysis();
        timing!("analysis invalidated");

        // Store result and show summary
        let deleted = result.deleted_count;
        let freed = result.bytes_freed;
        let failed = result.failed_count;

        self.last_clean_result = Some(result);
        self.pending_clean_items.clear();
        self.pending_clean_size = 0;
        self.deletion_progress = None;

        if failed == 0 {
            self.status_msg = format!(
                "Cleaned {} items, freed {}",
                deleted,
                crate::utils::format_bytes(freed)
            );
        } else {
            self.status_msg = format!(
                "Cleaned {} items, {} failed, freed {}",
                deleted,
                failed,
                crate::utils::format_bytes(freed)
            );
            self.error_msg = Some(format!("{} items failed to delete", failed));
        }

        // Skip cache save after deletion - it's slow for large trees and will be saved on quit/refresh
        // The tree is already updated in memory, so the UI is correct
        timing!("skipping cache save (will save on quit)");
        self.state = AppState::Browsing;
        timing!("finish_deletion complete");
    }

    /// Cancel the pending cleaning operation
    pub fn cancel_cleaning(&mut self) {
        self.pending_clean_items.clear();
        self.pending_clean_size = 0;
        self.state = AppState::Browsing;
        self.status_msg = "Cleaning cancelled".to_string();
    }

    /// Find entry size from the tree by path
    fn find_entry_size(&self, path: &PathBuf) -> Option<u64> {
        fn search_entry(entry: &crate::scanner::Entry, path: &PathBuf) -> Option<u64> {
            if &entry.path == path {
                return Some(entry.size);
            }
            for child in &entry.children {
                if let Some(size) = search_entry(child, path) {
                    return Some(size);
                }
            }
            None
        }
        search_entry(&self.root, path)
    }

    /// Remove specific entries from the tree by path (fast - no filesystem checks)
    fn remove_specific_entries(&mut self, paths: &[PathBuf]) {
        use std::collections::HashSet;
        let path_set: HashSet<_> = paths.iter().collect();

        fn remove_from_children(children: &mut Vec<crate::scanner::Entry>, paths: &HashSet<&PathBuf>) -> bool {
            let before_len = children.len();
            children.retain(|entry| !paths.contains(&entry.path));
            let removed = children.len() != before_len;

            // Recursively check subdirectories
            for child in children.iter_mut() {
                if child.is_dir && remove_from_children(&mut child.children, paths) {
                    // Update counts after removal
                    child.file_count = child.children.iter().map(|c| c.file_count + if c.is_dir { 0 } else { 1 }).sum();
                    child.dir_count = child.children.iter().map(|c| c.dir_count + if c.is_dir { 1 } else { 0 }).sum();
                    child.size = child.children.iter().map(|c| c.size).sum();
                }
            }
            removed
        }

        remove_from_children(&mut self.root.children, &path_set);

        // Update root counts
        self.root.file_count = self.root.children.iter().map(|c| c.file_count + if c.is_dir { 0 } else { 1 }).sum();
        self.root.dir_count = self.root.children.iter().map(|c| c.dir_count + if c.is_dir { 1 } else { 0 }).sum();
        self.root.size = self.root.children.iter().map(|c| c.size).sum();
    }

    /// Remove entries that no longer exist from the tree (slow - checks filesystem)
    #[allow(dead_code)]
    fn remove_deleted_entries(&mut self) {
        remove_deleted_from_children(&mut self.root.children);
        // Update root counts
        self.root.file_count = self.root.children.iter().map(|c| c.file_count + if c.is_dir { 0 } else { 1 }).sum();
        self.root.dir_count = self.root.children.iter().map(|c| c.dir_count + if c.is_dir { 1 } else { 0 }).sum();
        self.root.size = self.root.children.iter().map(|c| c.size).sum();
    }

    /// Get directory size (helper)
    fn get_dir_size(&self, path: &std::path::Path) -> u64 {
        let mut size = 0;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    size += path.metadata().map(|m| m.len()).unwrap_or(0);
                } else if path.is_dir() {
                    size += self.get_dir_size(&path);
                }
            }
        }
        size
    }

    /// Clean selected item from analysis view
    pub fn clean_analysis_selected(&mut self) {
        let item: Option<(PathBuf, u64)> = match self.state {
            AppState::LargeFilesView => {
                self.large_files.as_ref().and_then(|files| {
                    files.get(self.analysis_selected).map(|f| (f.path.clone(), f.size))
                })
            }
            AppState::CacheView => {
                self.system_caches.as_ref().and_then(|caches| {
                    caches.get(self.analysis_selected).map(|c| {
                        let size = c.size.unwrap_or(0);
                        (c.path.clone(), size)
                    })
                })
            }
            AppState::DuplicateAnalysis => {
                // For duplicates, get the first duplicate (not original) from selected group
                self.duplicate_result.as_ref().and_then(|result| {
                    result.groups.get(self.analysis_selected).and_then(|g| {
                        g.files.get(1).map(|path| (path.clone(), g.size))
                    })
                })
            }
            _ => None,
        };

        if let Some((path, size)) = item {
            self.pending_clean_items = vec![(path, size)];
            self.pending_clean_size = size;
            self.state = AppState::CleaningConfirm;
        } else {
            self.status_msg = "No item selected".to_string();
        }
    }
}

/// Recursively remove entries that no longer exist from children
fn remove_deleted_from_children(children: &mut Vec<crate::scanner::Entry>) {
    // Remove entries that no longer exist
    children.retain(|entry| entry.path.exists());

    // Recursively process remaining directories
    for child in children.iter_mut() {
        if child.is_dir {
            remove_deleted_from_children(&mut child.children);
            // Update counts
            child.file_count = child.children.iter().map(|c| c.file_count + if c.is_dir { 0 } else { 1 }).sum();
            child.dir_count = child.children.iter().map(|c| c.dir_count + if c.is_dir { 1 } else { 0 }).sum();
            child.size = child.children.iter().map(|c| c.size).sum();
        }
    }
}
