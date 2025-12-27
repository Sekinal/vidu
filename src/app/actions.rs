//! Application action handlers

use super::state::{App, AppState, SortMode, SortOrder};
use crate::scanner::{
    directory_preview, file_info_preview, read_file_preview, refresh_children, scan_with_progress,
    ScanOptions,
};
use crate::utils::{format_bytes, format_time, is_text_file};

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
        let current = self.current_view();
        let selected = self.table_state.selected().unwrap_or(0);
        if selected < current.children.len().saturating_sub(1) {
            self.table_state.select(Some(selected + 1));
        }
    }

    pub fn go_to_top(&mut self) {
        if !self.current_view().children.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn go_to_bottom(&mut self) {
        let len = self.current_view().children.len();
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
        let current = self.current_view();
        let selected = self.table_state.selected().unwrap_or(0);
        let jump = self.visible_rows();
        let new_pos = (selected + jump).min(current.children.len().saturating_sub(1));
        self.table_state.select(Some(new_pos));
    }

    pub fn enter_dir(&mut self) {
        let Some(selected) = self.table_state.selected() else {
            return;
        };

        let current = self.current_view();
        if selected >= current.children.len() {
            return;
        }

        let child = &current.children[selected];
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

        self.nav_stack.push(selected);
        self.table_state.select(Some(0));
        self.apply_sort();
    }

    pub fn go_back(&mut self) {
        if !self.nav_stack.is_empty() {
            let prev_idx = self.nav_stack.pop().unwrap();
            self.table_state.select(Some(prev_idx));
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
        let Some(selected) = self.table_state.selected() else {
            self.state = AppState::Browsing;
            return;
        };

        // Get item info
        let (path, is_dir, size, file_count) = {
            let view = self.current_view();
            if selected >= view.children.len() {
                self.state = AppState::Browsing;
                return;
            }
            let item = &view.children[selected];
            (item.path.clone(), item.is_dir, item.size, item.file_count)
        };

        // Perform deletion
        let result = if is_dir {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        match result {
            Ok(()) => {
                let new_len;
                {
                    let current = self.current_view_mut();
                    current.children.remove(selected);
                    current.size = current.size.saturating_sub(size);
                    current.file_count = current.file_count.saturating_sub(file_count);
                    new_len = current.children.len();
                }

                // Update disk available space (approximate)
                self.disk_available = self.disk_available.saturating_add(size);

                // Adjust selection
                if selected >= new_len && new_len > 0 {
                    self.table_state.select(Some(new_len - 1));
                } else if new_len == 0 {
                    self.table_state.select(None);
                }

                // Save cache
                self.save_to_cache();

                self.status_msg = format!(
                    "Deleted: {} ({})",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    format_bytes(size)
                );
            }
            Err(e) => {
                self.error_msg = Some(format!("Delete failed: {}", e));
            }
        }

        self.state = AppState::Browsing;
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
        self.refresh_current();
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
}
