# 📦 Codebase Context: vidu
> Generated on 2025-11-29 16:10:51 | Files: 19 | Tokens: ~17976

## 🌲 Project Structure
```text
📂 vidu/
├── .gitignore
├── Cargo.toml
├── src/
│   ├── app/
│   │   ├── actions.rs
│   │   ├── mod.rs
│   │   ├── state.rs
│   ├── cache.rs
│   ├── main.rs
│   ├── scanner/
│   │   ├── mod.rs
│   ├── ui/
│   │   ├── components/
│   │   │   ├── footer.rs
│   │   │   ├── header.rs
│   │   │   ├── help.rs
│   │   │   ├── mod.rs
│   │   │   ├── popups.rs
│   │   │   ├── preview.rs
│   │   │   ├── table.rs
│   │   ├── mod.rs
│   │   ├── render.rs
│   │   ├── theme.rs
│   ├── utils/
│   │   ├── mod.rs
```

## 📄 File Contents

### `.gitignore`
_Language: text | Lines: 1 | Tokens: ~2_
```text
/target
```
---

### `Cargo.toml`
_Language: toml | Lines: 24 | Tokens: ~152_
```toml
[package]
name = "vidu"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1.0.100"
bincode = { version = "2.0.1", features = ["serde"] }
chrono = "0.4.42"
clap = { version = "4.5.53", features = ["derive"] }
crossterm = { version = "0.29.0", features = ["event-stream"] }
directories = "6.0.0"
humansize = "2.1.3"
lz4_flex = "0.12.0"
md5 = "0.8.0"
ratatui = "0.29.0"
rayon = "1.11.0"
serde = { version = "1.0.228", features = ["derive"] }
sysinfo = "0.37.2"
textwrap = "0.16.2"
thiserror = "2.0.17"
tokio = { version = "1.48.0", features = ["full"] }
tokio-util = "0.7.17"
unicode-width = "0.2.0"
```
---

### `src/app/actions.rs`
_Language: rust | Lines: 376 | Tokens: ~3076_
```rust
//! Application action handlers

use super::state::{App, AppState, SortMode, SortOrder};
use crate::cache::CacheManager;
use crate::scanner::{read_file_preview, Entry};
use crate::utils::is_text_file;

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
        self.status_msg = format!(
            "Sort order: {}",
            match self.sort_order {
                SortOrder::Ascending => "Ascending",
                SortOrder::Descending => "Descending",
            }
        );
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
                // Scope to limit mutable borrow of self via current_view_mut
                {
                    // Update in-memory tree
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
                let _ = CacheManager::save(&self.root);
                
                self.status_msg = format!(
                    "Deleted: {} ({})",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    crate::utils::format_bytes(size)
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
        
        let show_hidden = self.show_hidden;
        let current = self.current_view_mut();
        current.refresh_children(show_hidden);
        
        self.apply_sort();
        
        // Update selection if out of bounds
        let len = self.current_view().children.len();
        if let Some(selected) = self.table_state.selected() {
            if selected >= len {
                self.table_state.select(if len > 0 { Some(len - 1) } else { None });
            }
        }
        
        let _ = CacheManager::save(&self.root);
        self.status_msg = "Refreshed".to_string();
    }
    
    pub fn full_rescan(&mut self) {
        self.status_msg = "Rescanning...".to_string();
        
        let path = self.original_path.clone();
        self.root = Entry::scan(path, self.show_hidden);
        self.nav_stack.clear();
        
        if !self.root.children.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
        
        let _ = CacheManager::save(&self.root);
        self.status_msg = "Rescan complete".to_string();
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
            self.preview_content = vec![
                format!("Directory: {}", item.name),
                "".to_string(),
                format!("Total size: {}", crate::utils::format_bytes(item.size)),
                format!("Files: {}", item.file_count),
                format!("Directories: {}", item.dir_count),
                format!("Modified: {}", crate::utils::format_time(item.modified)),
                "".to_string(),
                "Press Enter to open this directory".to_string(),
                "Press 'p' or Esc to close preview".to_string(),
            ];
        } else if is_text_file(&item.name) {
            match read_file_preview(&item.path, 500, 1_000_000) {
                Ok(lines) => {
                    self.preview_content = lines;
                }
                Err(e) => {
                    self.preview_content = vec![
                        format!("Cannot read file: {}", e),
                        "".to_string(),
                        "Press 'p' to close preview".to_string(),
                    ];
                }
            }
        } else {
            self.preview_content = vec![
                format!("File: {}", item.name),
                "".to_string(),
                format!("Size: {}", crate::utils::format_bytes(item.size)),
                format!("Modified: {}", crate::utils::format_time(item.modified)),
                "".to_string(),
                "Binary or unknown file type - cannot preview".to_string(),
                "".to_string(),
                "Press 'p' to close preview".to_string(),
            ];
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
        
        // Collect results to avoid keeping reference to self.current_view()
        let results: Vec<usize> = {
            let current = self.current_view();
            current.children
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
```
---

### `src/app/mod.rs`
_Language: rust | Lines: 6 | Tokens: ~27_
```rust
//! Application module

mod state;
mod actions;

pub use state::{App, AppState, ViewMode, SortMode, SortOrder};
```
---

### `src/app/state.rs`
_Language: rust | Lines: 411 | Tokens: ~3072_
```rust
//! Application state management

use crate::cache::CacheManager;
use crate::scanner::{Entry, ScanProgress};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::TableState};
use std::{
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use sysinfo::Disks;

/// Current application state/mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Browsing,
    DeleteConfirm,
    Preview,
    Help,
    Search,
    Scanning,
}

/// What view we're displaying
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tree,
    Flat, // Could be extended for flat view of all files
}

/// Sort mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Size,
    Name,
    Modified,
    Count,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Size => "Size",
            SortMode::Name => "Name",
            SortMode::Modified => "Modified",
            SortMode::Count => "Items",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            SortMode::Size => SortMode::Name,
            SortMode::Name => SortMode::Modified,
            SortMode::Modified => SortMode::Count,
            SortMode::Count => SortMode::Size,
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// Main application state
pub struct App {
    // Core state
    pub root: Entry,
    pub original_path: PathBuf,
    pub nav_stack: Vec<usize>,
    pub table_state: TableState,
    
    // UI state
    pub state: AppState,
    pub view_mode: ViewMode,
    pub sort_mode: SortMode,
    pub sort_order: SortOrder,
    pub show_hidden: bool,
    
    // Status
    pub status_msg: String,
    pub error_msg: Option<String>,
    
    // Disk info
    pub disk_total: u64,
    pub disk_available: u64,
    
    // Preview state
    pub preview_content: Vec<String>,
    pub preview_scroll: usize,
    
    // Search state
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
    
    // Scan progress (for background scanning)
    pub scan_progress: Option<Arc<ScanProgress>>,
    
    // Marks for multi-select
    pub marked_items: std::collections::HashSet<PathBuf>,
    
    // Config
    pub force_fresh: bool,
}

impl App {
    pub fn new(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<Self> {
        // Get disk info
        let disks = Disks::new_with_refreshed_list();
        let mut disk_total = 0u64;
        let mut disk_available = 0u64;
        
        for disk in &disks {
            if path.starts_with(disk.mount_point()) {
                disk_total = disk.total_space();
                disk_available = disk.available_space();
                break;
            }
        }
        
        // Try to load from cache unless force_fresh
        let (root, from_cache) = if !force_fresh {
            match CacheManager::load(&path) {
                Ok(entry) => (entry, true),
                Err(_) => (Entry::scan(path.clone(), show_hidden), false),
            }
        } else {
            (Entry::scan(path.clone(), show_hidden), false)
        };
        
        let mut app = Self {
            root,
            original_path: path,
            nav_stack: Vec::new(),
            table_state: TableState::default(),
            state: AppState::Browsing,
            view_mode: ViewMode::Tree,
            sort_mode: SortMode::Size,
            sort_order: SortOrder::Descending,
            show_hidden,
            status_msg: if from_cache {
                "Loaded from cache".to_string()
            } else {
                "Scanned".to_string()
            },
            error_msg: None,
            disk_total,
            disk_available,
            preview_content: Vec::new(),
            preview_scroll: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            scan_progress: None,
            marked_items: std::collections::HashSet::new(),
            force_fresh,
        };
        
        // Select first item if available
        if !app.root.children.is_empty() {
            app.table_state.select(Some(0));
        }
        
        // Save cache if we just scanned
        if !from_cache {
            let _ = CacheManager::save(&app.root);
        }
        
        Ok(app)
    }
    
    /// Main event loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Render
            terminal.draw(|f| ui::render::render(f, self))?;
            
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.handle_key(key.code, key.modifiers) {
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle key press, returns true if should quit
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Clear error message on any key
        self.error_msg = None;
        
        match self.state {
            AppState::Browsing => self.handle_browsing_key(code, modifiers),
            AppState::DeleteConfirm => self.handle_delete_confirm_key(code),
            AppState::Preview => self.handle_preview_key(code),
            AppState::Help => self.handle_help_key(code),
            AppState::Search => self.handle_search_key(code),
            AppState::Scanning => false, // No input during scan
        }
    }
    
    fn handle_browsing_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        match code {
            // Quit
            KeyCode::Char('q') => return true,
            KeyCode::Esc => {
                if !self.nav_stack.is_empty() {
                    self.go_back();
                } else {
                    return true;
                }
            }
            
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('g') if modifiers.contains(KeyModifiers::NONE) => self.go_to_top(),
            KeyCode::Char('G') => self.go_to_bottom(),
            KeyCode::Home => self.go_to_top(),
            KeyCode::End => self.go_to_bottom(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::PageUp => self.page_up(),
            
            // Enter directory
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.enter_dir(),
            
            // Go back
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => self.go_back(),
            
            // Delete
            KeyCode::Char('d') => self.request_delete(),
            KeyCode::Delete => self.request_delete(),
            
            // Mark/unmark for multi-select
            KeyCode::Char(' ') => self.toggle_mark(),
            
            // Sort
            KeyCode::Char('s') => self.cycle_sort(),
            KeyCode::Char('S') => self.toggle_sort_order(),
            
            // Refresh
            KeyCode::Char('r') => self.refresh_current(),
            KeyCode::Char('R') => self.full_rescan(),
            
            // Preview
            KeyCode::Char('p') => self.toggle_preview(),
            
            // Help
            KeyCode::Char('?') => self.state = AppState::Help,
            
            // Search
            KeyCode::Char('/') => {
                self.search_query.clear();
                self.state = AppState::Search;
            }
            
            // Toggle hidden files
            KeyCode::Char('.') => self.toggle_hidden(),
            
            // Go to root
            KeyCode::Char('~') => self.go_to_root(),
            
            _ => {}
        }
        
        false
    }
    
    fn handle_delete_confirm_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.confirm_delete();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                self.state = AppState::Browsing;
            }
            _ => {}
        }
        false
    }
    
    fn handle_preview_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('p') | KeyCode::Esc | KeyCode::Char('q') => {
                self.state = AppState::Browsing;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.preview_scroll < self.preview_content.len().saturating_sub(1) {
                    self.preview_scroll += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.preview_scroll = self.preview_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.preview_scroll = (self.preview_scroll + 20)
                    .min(self.preview_content.len().saturating_sub(1));
            }
            KeyCode::PageUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(20);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.preview_scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.preview_scroll = self.preview_content.len().saturating_sub(1);
            }
            _ => {}
        }
        false
    }
    
    fn handle_help_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
                self.state = AppState::Browsing;
            }
            _ => {}
        }
        false
    }
    
    fn handle_search_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => {
                self.state = AppState::Browsing;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.execute_search();
                self.state = AppState::Browsing;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            _ => {}
        }
        false
    }
    
    /// Get current view entry (the directory we're viewing)
    pub fn current_view(&self) -> &Entry {
        let mut current = &self.root;
        for &idx in &self.nav_stack {
            if idx < current.children.len() {
                current = &current.children[idx];
            }
        }
        current
    }
    
    /// Get current view entry mutably
    pub fn current_view_mut(&mut self) -> &mut Entry {
        let mut current = &mut self.root;
        for &idx in &self.nav_stack {
            current = &mut current.children[idx];
        }
        current
    }
    
    /// Get selected item in current view
    pub fn selected_item(&self) -> Option<&Entry> {
        let view = self.current_view();
        self.table_state.selected()
            .and_then(|idx| view.children.get(idx))
    }
    
    /// Get breadcrumb path segments
    pub fn breadcrumbs(&self) -> Vec<String> {
        let mut crumbs = vec![self.root.name.clone()];
        let mut current = &self.root;
        
        for &idx in &self.nav_stack {
            if idx < current.children.len() {
                current = &current.children[idx];
                crumbs.push(current.name.clone());
            }
        }
        
        crumbs
    }
    
    /// Get visible table height
    pub fn visible_rows(&self) -> usize {
        20 // Default, should be calculated from terminal size
    }
}
```
---

### `src/cache.rs`
_Language: rust | Lines: 49 | Tokens: ~422_
```rust
use crate::scanner::Entry;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CacheManager;

impl CacheManager {
    fn get_cache_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "vidu", "vidu")
            .context("Could not determine cache directory")?;
        let cache_dir = proj_dirs.cache_dir();
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir)?;
        }
        Ok(cache_dir.to_path_buf())
    }

    fn get_cache_file(path: &Path) -> Result<PathBuf> {
        let hash = md5::compute(path.to_string_lossy().as_bytes());
        let cache_dir = Self::get_cache_dir()?;
        Ok(cache_dir.join(format!("{:x}.bin", hash)))
    }

    pub fn save(entry: &Entry) -> Result<()> {
        let cache_file = Self::get_cache_file(&entry.path)?;
        let config = bincode::config::standard();
        let data = bincode::serde::encode_to_vec(entry, config)?;
        let compressed = lz4_flex::compress_prepend_size(&data);
        fs::write(cache_file, compressed)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Entry> {
        let cache_file = Self::get_cache_file(path)?;
        if !cache_file.exists() {
            anyhow::bail!("Cache not found");
        }
        
        let compressed = fs::read(cache_file)?;
        let data = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;
            
        let config = bincode::config::standard();
        let (entry, _): (Entry, usize) = bincode::serde::decode_from_slice(&data, config)?;
        Ok(entry)
    }
}
```
---

### `src/main.rs`
_Language: rust | Lines: 98 | Tokens: ~615_
```rust
//! vidu - A blazingly fast disk usage analyzer
//!
//! Usage:
//!   vidu [PATH]            Analyze the given path (default: current directory)
//!   vidu --help            Show help
//!   vidu --version         Show version

mod app;
mod cache;
mod scanner;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, path::PathBuf, panic};

/// vidu - Blazingly fast disk usage analyzer
#[derive(Parser, Debug)]
#[command(name = "vidu", version, about, long_about = None)]
struct Args {
    /// Path to analyze (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Force a fresh scan, ignoring cache
    #[arg(short, long)]
    fresh: bool,

    /// Show hidden files
    #[arg(short = 'H', long)]
    hidden: bool,
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();
    
    // Resolve and validate path
    let path = args.path.canonicalize().unwrap_or_else(|_| {
        eprintln!("Error: Cannot access path '{}'", args.path.display());
        std::process::exit(1);
    });

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    // Setup panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    setup_terminal()?;
    
    // Run app
    let result = run_app(path, args.fresh, args.hidden);
    
    // Restore terminal
    restore_terminal()?;
    
    result
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new(path, force_fresh, show_hidden)?;
    
    // Run the main loop
    app.run(&mut terminal)?;
    
    Ok(())
}
```
---

### `src/scanner/mod.rs`
_Language: rust | Lines: 321 | Tokens: ~2315_
```rust
//! Directory scanning with parallel processing

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    sync::Arc,
    time::SystemTime,
};

/// Represents a file or directory entry with size information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<Entry>,
    pub modified: Option<SystemTime>,
    pub file_count: usize,
    pub dir_count: usize,
    #[serde(skip)]
    pub error: Option<String>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            name: String::new(),
            size: 0,
            path: PathBuf::new(),
            is_dir: false,
            children: Vec::new(),
            modified: None,
            file_count: 0,
            dir_count: 0,
            error: None,
        }
    }
}

/// Progress tracking during scan
#[derive(Default)]
pub struct ScanProgress {
    pub files_scanned: AtomicUsize,
    pub dirs_scanned: AtomicUsize,
    pub bytes_scanned: AtomicU64,
    pub current_path: std::sync::RwLock<String>,
    pub is_complete: AtomicBool,
    pub cancelled: AtomicBool,
}

impl ScanProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    
    pub fn files(&self) -> usize {
        self.files_scanned.load(Ordering::Relaxed)
    }
    
    pub fn dirs(&self) -> usize {
        self.dirs_scanned.load(Ordering::Relaxed)
    }
    
    pub fn bytes(&self) -> u64 {
        self.bytes_scanned.load(Ordering::Relaxed)
    }
    
    pub fn current(&self) -> String {
        self.current_path.read().unwrap().clone()
    }
    
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Entry {
    /// Scan a path and build the entry tree
    pub fn scan(path: PathBuf, show_hidden: bool) -> Self {
        Self::scan_with_progress(path, show_hidden, None)
    }
    
    /// Scan with progress tracking
    pub fn scan_with_progress(
        path: PathBuf,
        show_hidden: bool,
        progress: Option<Arc<ScanProgress>>,
    ) -> Self {
        Self::scan_internal(path, show_hidden, &progress, 0)
    }
    
    fn scan_internal(
        path: PathBuf,
        show_hidden: bool,
        progress: &Option<Arc<ScanProgress>>,
        depth: usize,
    ) -> Self {
        // Check for cancellation
        if let Some(p) = progress {
            if p.is_cancelled() {
                return Entry::default();
            }
        }
        
        let metadata = fs::symlink_metadata(&path);
        let name = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("."))
            .to_string_lossy()
            .to_string();

        let mut entry = Entry {
            name: name.clone(),
            size: 0,
            path: path.clone(),
            is_dir: false,
            children: Vec::new(),
            modified: None,
            file_count: 0,
            dir_count: 0,
            error: None,
        };

        let meta = match metadata {
            Ok(m) => m,
            Err(e) => {
                entry.error = Some(format!("Access denied: {}", e));
                return entry;
            }
        };

        entry.modified = meta.modified().ok();

        // Handle symlinks - don't follow them to avoid infinite loops
        if meta.is_symlink() {
            entry.size = meta.len();
            entry.file_count = 1;
            return entry;
        }

        if meta.is_dir() {
            entry.is_dir = true;
            entry.dir_count = 1;
            
            // Update progress
            if let Some(p) = progress {
                p.dirs_scanned.fetch_add(1, Ordering::Relaxed);
                if depth < 3 {
                    if let Ok(mut current) = p.current_path.write() {
                        *current = path.to_string_lossy().to_string();
                    }
                }
            }

            // Read directory entries
            let entries: Vec<_> = match fs::read_dir(&path) {
                Ok(rd) => rd
                    .filter_map(|r| r.ok())
                    .filter(|e| {
                        if show_hidden {
                            true
                        } else {
                            !e.file_name().to_string_lossy().starts_with('.')
                        }
                    })
                    .collect(),
                Err(e) => {
                    entry.error = Some(format!("Cannot read directory: {}", e));
                    return entry;
                }
            };

            // Parallel scan for children (only at shallow depths to avoid too many threads)
            if depth < 4 && entries.len() > 1 {
                entry.children = entries
                    .par_iter()
                    .map(|dir_entry| {
                        Self::scan_internal(
                            dir_entry.path(),
                            show_hidden,
                            progress,
                            depth + 1,
                        )
                    })
                    .collect();
            } else {
                entry.children = entries
                    .iter()
                    .map(|dir_entry| {
                        Self::scan_internal(
                            dir_entry.path(),
                            show_hidden,
                            progress,
                            depth + 1,
                        )
                    })
                    .collect();
            }

            // Aggregate stats
            for child in &entry.children {
                entry.size += child.size;
                entry.file_count += child.file_count;
                entry.dir_count += child.dir_count;
            }
            
            // Add directory's own size
            entry.size += meta.len();

            // Sort by size descending
            entry.children.sort_by(|a, b| b.size.cmp(&a.size));
        } else {
            // File
            entry.size = meta.len();
            entry.file_count = 1;
            
            if let Some(p) = progress {
                p.files_scanned.fetch_add(1, Ordering::Relaxed);
                p.bytes_scanned.fetch_add(entry.size, Ordering::Relaxed);
            }
        }

        entry
    }

    /// Refresh only this entry's children (faster than full rescan)
    pub fn refresh_children(&mut self, show_hidden: bool) {
        if !self.is_dir {
            return;
        }
        let fresh_node = Entry::scan(self.path.clone(), show_hidden);
        self.children = fresh_node.children;
        self.size = fresh_node.size;
        self.file_count = fresh_node.file_count;
        self.dir_count = fresh_node.dir_count;
        self.error = fresh_node.error;
    }

    /// Delete this entry from disk
    pub fn delete_from_disk(&self) -> Result<()> {
        if self.is_dir {
            fs::remove_dir_all(&self.path)?;
        } else {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
    
    /// Get total items (files + dirs)
    pub fn total_items(&self) -> usize {
        self.file_count + self.dir_count
    }
    
    /// Check if entry has an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

/// Read a preview of file contents
pub fn read_file_preview(path: &PathBuf, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    use std::io::{BufRead, BufReader};
    
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    
    // Don't try to preview very large files
    if metadata.len() > max_bytes as u64 {
        return Ok(vec![
            format!("File too large to preview ({} bytes)", metadata.len()),
            "".to_string(),
            "Press 'p' to close preview".to_string(),
        ]);
    }
    
    let reader = BufReader::new(file);
    let mut lines = Vec::with_capacity(max_lines);
    
    for (i, line) in reader.lines().enumerate() {
        if i >= max_lines {
            lines.push(format!("... ({} more lines)", "truncated"));
            break;
        }
        
        match line {
            Ok(l) => {
                // Check for binary content
                if l.chars().any(|c| c.is_control() && c != '\t' && c != '\n') {
                    return Ok(vec![
                        "Binary file - cannot preview".to_string(),
                        "".to_string(),
                        "Press 'p' to close preview".to_string(),
                    ]);
                }
                lines.push(l);
            }
            Err(_) => {
                return Ok(vec![
                    "Binary file or encoding error - cannot preview".to_string(),
                    "".to_string(), 
                    "Press 'p' to close preview".to_string(),
                ]);
            }
        }
    }
    
    if lines.is_empty() {
        lines.push("(empty file)".to_string());
    }
    
    Ok(lines)
}
```
---

### `src/ui/components/footer.rs`
_Language: rust | Lines: 79 | Tokens: ~640_
```rust
//! Footer component with status and key hints

use crate::app::{App, AppState};
use crate::ui::theme::{styles, Theme};
use ratatui::{prelude::*, widgets::*};

/// Render the footer with status and key hints
pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    
    // Status message (with appropriate color)
    let status_style = if app.error_msg.is_some() {
        styles::danger()
    } else if app.status_msg.contains("Scanning") || app.status_msg.contains("Refreshing") {
        styles::warning()
    } else {
        styles::accent()
    };
    
    let status_text = app.error_msg.as_ref().unwrap_or(&app.status_msg);
    spans.push(Span::styled(format!(" {} ", status_text), status_style));
    
    // Separator
    spans.push(Span::styled(" │ ", styles::dim()));
    
    // Sort mode indicator
    let sort_indicator = format!(
        "Sort:{} {}",
        app.sort_mode.label(),
        match app.sort_order {
            crate::app::SortOrder::Ascending => "↑",
            crate::app::SortOrder::Descending => "↓",
        }
    );
    spans.push(Span::styled(sort_indicator, styles::dim()));
    
    // Key hints
    let keys: &[(&str, &str)] = match app.state {
        AppState::Browsing => &[
            ("↑↓", "nav"),
            ("↵", "open"),
            ("d", "del"),
            ("p", "preview"),
            ("s", "sort"),
            ("r", "refresh"),
            ("?", "help"),
            ("q", "quit"),
        ],
        AppState::DeleteConfirm => &[("y", "confirm"), ("n", "cancel")],
        AppState::Preview => &[("↑↓", "scroll"), ("p/Esc", "close")],
        AppState::Help => &[("Esc", "close")],
        AppState::Search => &[("↵", "search"), ("Esc", "cancel")],
        AppState::Scanning => &[],
    };
    
    for (key, desc) in keys {
        spans.push(Span::styled(" │ ", styles::dim()));
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(":{}", desc), styles::dim()));
    }
    
    // Marked items count (if any)
    if !app.marked_items.is_empty() {
        spans.push(Span::styled(" │ ", styles::dim()));
        spans.push(Span::styled(
            format!("{} marked", app.marked_items.len()),
            styles::marked(),
        ));
    }
    
    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(Theme::BG));
    
    f.render_widget(footer, area);
}
```
---

### `src/ui/components/header.rs`
_Language: rust | Lines: 71 | Tokens: ~539_
```rust
//! Header component with disk gauge and breadcrumbs

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::{format_bytes, truncate_str};
use ratatui::{prelude::*, widgets::*};

/// Render the disk usage gauge
pub fn render_disk_gauge(f: &mut Frame, app: &App, area: Rect) {
    let used = app.disk_total.saturating_sub(app.disk_available);
    let ratio = if app.disk_total > 0 {
        used as f64 / app.disk_total as f64
    } else {
        0.0
    };
    
    let color = Theme::bar_color(ratio * 100.0);
    
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border())
                .title(" 💾 Disk Usage ")
                .title_style(styles::accent()),
        )
        .gauge_style(Style::default().fg(color).bg(Theme::BG))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(format!(
            "{} / {} ({:.1}% used)",
            format_bytes(used),
            format_bytes(app.disk_total),
            ratio * 100.0
        ));
    
    f.render_widget(gauge, area);
}

/// Render the breadcrumb navigation path
pub fn render_breadcrumbs(f: &mut Frame, app: &App, area: Rect) {
    let current = app.current_view();
    let path_str = current.path.to_string_lossy();
    
    // Build breadcrumb spans
    let mut spans = vec![Span::styled(" 📂 ", styles::directory())];
    
    let max_width = area.width.saturating_sub(10) as usize;
    let display_path = truncate_str(&path_str, max_width);
    
    spans.push(Span::styled(
        display_path,
        Style::default().fg(Theme::FG).add_modifier(Modifier::BOLD),
    ));
    
    // Add item count
    let count_str = format!(
        " │ {} items ({} files, {} dirs)",
        current.children.len(),
        current.file_count,
        current.dir_count
    );
    spans.push(Span::styled(count_str, styles::dim()));
    
    let breadcrumb = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles::border()),
    );
    
    f.render_widget(breadcrumb, area);
}
```
---

### `src/ui/components/help.rs`
_Language: rust | Lines: 95 | Tokens: ~762_
```rust
//! Help screen component

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::centered_rect;
use ratatui::{prelude::*, widgets::*};

/// Render help screen
pub fn render_help(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 80, f.area());
    
    f.render_widget(Clear, area);
    
    let help_sections = vec![
        ("Navigation", vec![
            ("↑/k, ↓/j", "Move up/down"),
            ("←/h, Backspace", "Go to parent directory"),
            ("→/l, Enter", "Enter directory / Preview file"),
            ("g, Home", "Go to first item"),
            ("G, End", "Go to last item"),
            ("PgUp, PgDn", "Page up/down"),
            ("~", "Go to scan root"),
        ]),
        ("Actions", vec![
            ("d, Delete", "Delete selected item"),
            ("Space", "Mark/unmark item"),
            ("r", "Refresh current directory"),
            ("R", "Full rescan from root"),
            ("p", "Preview file contents"),
        ]),
        ("View", vec![
            ("s", "Cycle sort mode (size/name/modified/count)"),
            ("S", "Toggle sort order (asc/desc)"),
            (".", "Toggle hidden files"),
            ("/", "Search in current directory"),
        ]),
        ("General", vec![
            ("?", "Show/hide this help"),
            ("q, Esc", "Quit / Go back / Close popup"),
        ]),
    ];
    
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  vidu - Disk Usage Analyzer",
        Style::default()
            .fg(Theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  ═══════════════════════════",
        styles::dim(),
    )]));
    lines.push(Line::from(""));
    
    for (section_name, keys) in help_sections {
        lines.push(Line::from(vec![Span::styled(
            format!("  ┌─ {} ", section_name),
            styles::accent(),
        )]));
        
        for (key, desc) in keys {
            lines.push(Line::from(vec![
                Span::raw("  │ "),
                Span::styled(
                    format!("{:>14}", key),
                    Style::default()
                        .fg(Theme::CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(desc, Style::default().fg(Theme::FG)),
            ]));
        }
        
        lines.push(Line::from(vec![Span::styled("  └", styles::dim())]));
        lines.push(Line::from(""));
    }
    
    lines.push(Line::from(vec![Span::styled(
        "  Press Esc or ? to close",
        styles::dim(),
    )]));
    
    let help = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles::accent())
            .title(" ❓ Help ")
            .title_style(styles::accent()),
    );
    
    f.render_widget(help, area);
}
```
---

### `src/ui/components/mod.rs`
_Language: rust | Lines: 8 | Tokens: ~28_
```rust
//! UI components

pub mod footer;
pub mod header;
pub mod help;
pub mod popups;
pub mod preview;
pub mod table;
```
---

### `src/ui/components/popups.rs`
_Language: rust | Lines: 121 | Tokens: ~888_
```rust
//! Popup dialogs (delete confirmation, etc.)

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::{centered_rect, format_bytes};
use ratatui::{prelude::*, widgets::*};

/// Render delete confirmation popup
pub fn render_delete_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    
    // Clear background
    f.render_widget(Clear, area);
    
    let Some(idx) = app.table_state.selected() else {
        return;
    };
    
    let current = app.current_view();
    let Some(item) = current.children.get(idx) else {
        return;
    };
    
    let item_type = if item.is_dir { "directory" } else { "file" };
    let warnings = if item.is_dir && item.file_count > 0 {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("⚠ This will delete {} files!", item.file_count),
                styles::danger(),
            )]),
        ]
    } else {
        vec![]
    };
    
    let mut text = vec![
        Line::from(""),
        Line::from(vec![Span::raw(format!(
            "Are you sure you want to delete this {}?",
            item_type
        ))]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name: ", styles::dim()),
            Span::styled(&item.name, styles::file()),
        ]),
        Line::from(vec![
            Span::styled("  Size: ", styles::dim()),
            Span::styled(format_bytes(item.size), styles::size()),
        ]),
        Line::from(vec![
            Span::styled("  Path: ", styles::dim()),
            Span::raw(item.path.to_string_lossy().to_string()),
        ]),
    ];
    
    text.extend(warnings);
    
    text.extend(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "⚠ This action cannot be undone!",
            styles::warning(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("y", styles::danger()),
            Span::raw(" to delete, "),
            Span::styled("n", styles::success()),
            Span::raw(" to cancel"),
        ]),
    ]);
    
    let popup = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::RED))
                .title(" ⚠ DELETE CONFIRMATION ")
                .title_style(styles::danger()),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(popup, area);
}

/// Render search input popup
pub fn render_search_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, f.area());
    
    f.render_widget(Clear, area);
    
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Search: ", styles::accent()),
            Span::styled(&app.search_query, Style::default().fg(Theme::FG)),
            Span::styled("▎", Style::default().fg(Theme::ACCENT)), // Cursor
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Enter to search, Esc to cancel",
            styles::dim(),
        )]),
    ];
    
    let popup = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::accent())
                .title(" 🔍 Search ")
                .title_style(styles::accent()),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(popup, area);
}
```
---

### `src/ui/components/preview.rs`
_Language: rust | Lines: 71 | Tokens: ~549_
```rust
//! File preview component

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::centered_rect;
use ratatui::{prelude::*, widgets::*};

/// Render file preview panel
pub fn render_preview(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());
    
    f.render_widget(Clear, area);
    
    let title = if let Some(item) = app.selected_item() {
        format!(" Preview: {} ", item.name)
    } else {
        " Preview ".to_string()
    };
    
    // Calculate visible lines
    let inner_height = area.height.saturating_sub(2) as usize;
    let total_lines = app.preview_content.len();
    let start = app.preview_scroll;
    let end = (start + inner_height).min(total_lines);
    
    let visible_lines: Vec<Line> = app.preview_content[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            // Line numbers
            let line_num = format!("{:>4} │ ", start + i + 1);
            let mut spans = vec![Span::styled(line_num, styles::dim())];
            
            // Syntax highlighting (basic)
            spans.push(Span::styled(line, Style::default().fg(Theme::FG)));
            
            Line::from(spans)
        })
        .collect();
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::accent())
        .title(title)
        .title_style(styles::accent());
    
    let paragraph = Paragraph::new(visible_lines).block(block);
    
    f.render_widget(paragraph, area);
    
    // Scroll indicator
    if total_lines > inner_height {
        let scroll_info = format!(
            " Lines {}-{} of {} ({}%) ",
            start + 1,
            end,
            total_lines,
            ((start as f64 / total_lines.saturating_sub(inner_height) as f64) * 100.0) as u32
        );
        
        let info_area = Rect {
            x: area.x + area.width.saturating_sub(scroll_info.len() as u16 + 2),
            y: area.y + area.height - 1,
            width: scroll_info.len() as u16,
            height: 1,
        };
        
        let info = Paragraph::new(scroll_info).style(styles::dim());
        f.render_widget(info, info_area);
    }
}
```
---

### `src/ui/components/table.rs`
_Language: rust | Lines: 164 | Tokens: ~1417_
```rust
//! Main table component for displaying entries

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::{create_smooth_bar, format_age, format_bytes, get_file_icon, truncate_str};
use ratatui::{prelude::*, widgets::*};

/// Render the main file/directory table
pub fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    // We need to calculate rows first to avoid double borrowing app
    // (current_view borrows app, and we need &mut app for table_state later)
    let (rows, total_items, selected_idx) = {
        let current = app.current_view();
        let parent_size = current.size as f64;
        let marked_items = &app.marked_items;
        
        let rows: Vec<Row> = current
            .children
            .iter()
            .enumerate()
            .map(|(_idx, item)| {
                let is_marked = marked_items.contains(&item.path);
                let icon = get_file_icon(&item.name, item.is_dir);
                
                // Name with icon
                let name_style = if is_marked {
                    styles::marked()
                } else if item.is_dir {
                    styles::directory()
                } else {
                    styles::file()
                };
                
                let mark_indicator = if is_marked { "◉ " } else { "  " };
                let error_indicator = if item.has_error() { " ⚠" } else { "" };
                
                // Calculate max name width (40% of area minus some padding)
                let max_name_width = (area.width as f64 * 0.35) as usize;
                let name_display = truncate_str(&item.name, max_name_width.saturating_sub(6));
                
                let name_cell = Cell::from(format!(
                    "{}{}  {}{}",
                    mark_indicator, icon, name_display, error_indicator
                ))
                .style(name_style);
                
                // Size bar
                let percentage = if parent_size > 0.0 {
                    (item.size as f64 / parent_size) * 100.0
                } else {
                    0.0
                };
                
                let bar = create_smooth_bar(percentage, 12);
                let bar_color = Theme::bar_color(percentage);
                let bar_cell = Cell::from(bar).style(Style::default().fg(bar_color));
                
                // Percentage
                let pct_cell = Cell::from(format!("{:>5.1}%", percentage))
                    .style(Style::default().fg(bar_color));
                
                // Items count
                let items_cell = Cell::from(format!(
                    "{:>6}",
                    if item.is_dir {
                        item.file_count.to_string()
                    } else {
                        "-".to_string()
                    }
                ))
                .style(styles::dim());
                
                // Size
                let size_str = format_bytes(item.size);
                let size_cell = Cell::from(format!("{:>9}", size_str)).style(styles::size());
                
                // Age
                let age_str = format_age(item.modified);
                let age_cell = Cell::from(format!("{:>12}", age_str)).style(styles::dim());
                
                Row::new(vec![
                    name_cell,
                    bar_cell,
                    pct_cell,
                    items_cell,
                    size_cell,
                    age_cell,
                ])
            })
            .collect();

        (rows, current.children.len(), app.table_state.selected().unwrap_or(0))
    };
    
    // Column widths
    let widths = [
        Constraint::Percentage(35),  // Name
        Constraint::Length(12),      // Bar
        Constraint::Length(7),       // Percent
        Constraint::Length(7),       // Items
        Constraint::Length(10),      // Size
        Constraint::Length(13),      // Age
    ];
    
    // Header
    let header = Row::new(vec![
        Cell::from(" Name"),
        Cell::from("Usage"),
        Cell::from("    %"),
        Cell::from(" Items"),
        Cell::from("     Size"),
        Cell::from("    Modified"),
    ])
    .style(styles::header())
    .height(1);
    
    // Empty state
    if rows.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("  📭 This directory is empty"),
            Line::from(""),
            Line::from("  Press Backspace to go back"),
        ])
        .style(styles::dim())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border()),
        );
        f.render_widget(empty_msg, area);
        return;
    }
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(styles::border()),
        )
        .row_highlight_style(styles::highlight())
        .highlight_symbol("▶ ");
    
    f.render_stateful_widget(table, area, &mut app.table_state);
    
    // Scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    
    let mut scrollbar_state = ScrollbarState::new(total_items)
        .position(selected_idx);
    
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
```
---

### `src/ui/mod.rs`
_Language: rust | Lines: 5 | Tokens: ~16_
```rust
//! UI module

pub mod components;
pub mod render;
pub mod theme;
```
---

### `src/ui/render.rs`
_Language: rust | Lines: 106 | Tokens: ~804_
```rust
//! Main render function

use crate::app::{App, AppState};
use crate::ui::components::{footer, header, help, popups, preview, table};
use crate::ui::theme::{styles, Theme};
use ratatui::{prelude::*, widgets::*};

/// Main render function
pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    
    // Clear with background color
    f.render_widget(
        Block::default().style(Style::default().bg(Theme::BG)),
        area,
    );
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Disk gauge
            Constraint::Length(3),  // Breadcrumbs
            Constraint::Min(5),     // Table
            Constraint::Length(1),  // Footer
        ])
        .split(area);
    
    // Render main components
    header::render_disk_gauge(f, app, chunks[0]);
    header::render_breadcrumbs(f, app, chunks[1]);
    table::render_table(f, app, chunks[2]);
    footer::render_footer(f, app, chunks[3]);
    
    // Render overlays based on state
    match app.state {
        AppState::DeleteConfirm => {
            popups::render_delete_popup(f, app);
        }
        AppState::Preview => {
            preview::render_preview(f, app);
        }
        AppState::Help => {
            help::render_help(f, app);
        }
        AppState::Search => {
            popups::render_search_popup(f, app);
        }
        AppState::Scanning => {
            render_scanning_overlay(f, app);
        }
        AppState::Browsing => {}
    }
}

/// Render scanning progress overlay
fn render_scanning_overlay(f: &mut Frame, app: &App) {
    let area = crate::utils::centered_rect(50, 25, f.area());
    
    f.render_widget(Clear, area);
    
    let progress_text = if let Some(ref progress) = app.scan_progress {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(" Scanning... ", styles::accent())]),
            Line::from(""),
            Line::from(vec![Span::raw(format!(
                " Files: {} | Dirs: {}",
                progress.files(),
                progress.dirs()
            ))]),
            Line::from(vec![Span::raw(format!(
                " Bytes: {}",
                crate::utils::format_bytes(progress.bytes())
            ))]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Current: ",
                styles::dim(),
            )]),
            Line::from(vec![Span::raw(crate::utils::truncate_str(
                &progress.current(),
                40,
            ))]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(" Scanning... ", styles::accent())]),
            Line::from(""),
            Line::from(vec![Span::raw(" Please wait...")]),
        ]
    };
    
    let popup = Paragraph::new(progress_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::accent())
                .title(" 🔄 Scanning ")
                .title_style(styles::accent()),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(popup, area);
}
```
---

### `src/ui/theme.rs`
_Language: rust | Lines: 110 | Tokens: ~773_
```rust
//! Color theme and styling constants

use ratatui::prelude::*;

/// Application color theme (Dracula-inspired)
pub struct Theme;

impl Theme {
    // Background colors
    pub const BG: Color = Color::Rgb(40, 42, 54);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(68, 71, 90);
    pub const BG_SELECTION: Color = Color::Rgb(68, 71, 90);
    
    // Foreground colors
    pub const FG: Color = Color::Rgb(248, 248, 242);
    pub const FG_DIM: Color = Color::Rgb(98, 114, 164);
    
    // Accent colors
    pub const ACCENT: Color = Color::Rgb(189, 147, 249);       // Purple
    pub const CYAN: Color = Color::Rgb(139, 233, 253);         // Cyan - directories
    pub const PINK: Color = Color::Rgb(255, 121, 198);         // Pink - files
    pub const GREEN: Color = Color::Rgb(80, 250, 123);         // Green - sizes
    pub const YELLOW: Color = Color::Rgb(241, 250, 140);       // Yellow
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);       // Orange - warning
    pub const RED: Color = Color::Rgb(255, 85, 85);            // Red - danger
    
    // Semantic colors
    pub const DIR: Color = Self::CYAN;
    pub const FILE: Color = Self::PINK;
    pub const SIZE: Color = Self::GREEN;
    pub const WARN: Color = Self::ORANGE;
    pub const DANGER: Color = Self::RED;
    pub const SUCCESS: Color = Self::GREEN;
    pub const INFO: Color = Self::CYAN;
    pub const MARKED: Color = Self::YELLOW;
    pub const ERROR: Color = Self::RED;
    
    // Bar gradient colors
    pub fn bar_color(percent: f64) -> Color {
        if percent > 80.0 {
            Self::DANGER
        } else if percent > 50.0 {
            Self::ORANGE
        } else if percent > 20.0 {
            Self::YELLOW
        } else {
            Self::GREEN
        }
    }
}

/// Common styles
pub mod styles {
    use super::*;
    
    pub fn normal() -> Style {
        Style::default().fg(Theme::FG).bg(Theme::BG)
    }
    
    pub fn highlight() -> Style {
        Style::default().fg(Theme::FG).bg(Theme::BG_SELECTION)
    }
    
    pub fn header() -> Style {
        Style::default()
            .fg(Theme::BG)
            .bg(Theme::CYAN)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn directory() -> Style {
        Style::default().fg(Theme::DIR)
    }
    
    pub fn file() -> Style {
        Style::default().fg(Theme::FILE)
    }
    
    pub fn size() -> Style {
        Style::default().fg(Theme::SIZE)
    }
    
    pub fn warning() -> Style {
        Style::default().fg(Theme::WARN)
    }
    
    pub fn danger() -> Style {
        Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD)
    }
    
    pub fn success() -> Style {
        Style::default().fg(Theme::SUCCESS)
    }
    
    pub fn dim() -> Style {
        Style::default().fg(Theme::FG_DIM)
    }
    
    pub fn marked() -> Style {
        Style::default().fg(Theme::MARKED).add_modifier(Modifier::BOLD)
    }
    
    pub fn border() -> Style {
        Style::default().fg(Theme::BG_HIGHLIGHT)
    }
    
    pub fn accent() -> Style {
        Style::default().fg(Theme::ACCENT)
    }
}
```
---

### `src/utils/mod.rs`
_Language: rust | Lines: 256 | Tokens: ~1879_
```rust
//! Utility functions and helpers

use chrono::{DateTime, Local};
use humansize::{format_size, DECIMAL};
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

/// Format bytes into human-readable size
pub fn format_bytes(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Format SystemTime into readable string
pub fn format_time(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => {
            let datetime: DateTime<Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "Unknown".to_string(),
    }
}

/// Get relative age string (e.g., "2 days ago", "3 months ago")
pub fn format_age(time: Option<SystemTime>) -> String {
    let Some(t) = time else {
        return "Unknown".to_string();
    };

    let now = SystemTime::now();
    let duration = match now.duration_since(t) {
        Ok(d) => d,
        Err(_) => return "Future".to_string(),
    };

    let secs = duration.as_secs();
    
    if secs < 60 {
        "Just now".to_string()
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else if secs < 86400 {
        format!("{} hours ago", secs / 3600)
    } else if secs < 604800 {
        format!("{} days ago", secs / 86400)
    } else if secs < 2592000 {
        format!("{} weeks ago", secs / 604800)
    } else if secs < 31536000 {
        format!("{} months ago", secs / 2592000)
    } else {
        format!("{} years ago", secs / 31536000)
    }
}

/// Truncate string to fit width with ellipsis
pub fn truncate_str(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "...".chars().take(max_width).collect()
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if current_width + char_width + 3 > max_width {
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        result.push_str("...");
        result
    }
}

/// Create a smooth progress bar using Unicode block characters
pub fn create_smooth_bar(percent: f64, width: usize) -> String {
    const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    
    let percent = percent.clamp(0.0, 100.0);
    let total_filled = (percent / 100.0) * width as f64;
    let full_blocks = total_filled.floor() as usize;
    let remainder = total_filled - full_blocks as f64;
    let partial_idx = (remainder * 8.0).round() as usize;
    
    let mut s = String::with_capacity(width);
    
    for _ in 0..full_blocks.min(width) {
        s.push(BLOCKS[8]);
    }
    
    if full_blocks < width {
        s.push(BLOCKS[partial_idx.min(8)]);
        for _ in 0..(width - full_blocks - 1) {
            s.push(' ');
        }
    }
    
    s
}

/// Get file type icon based on extension or name
pub fn get_file_icon(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return match name.to_lowercase().as_str() {
            ".git" => "",
            "node_modules" => "",
            "target" => "",
            ".cache" | "cache" => "",
            "downloads" => "",
            "documents" => "",
            "pictures" | "images" => "",
            "music" => "",
            "videos" => "",
            _ => "",
        };
    }
    
    // Get extension
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    
    match ext.as_str() {
        // Programming
        "rs" => "",
        "py" => "",
        "js" | "mjs" | "cjs" => "",
        "ts" | "tsx" => "",
        "go" => "",
        "c" | "h" => "",
        "cpp" | "hpp" | "cc" => "",
        "java" => "",
        "rb" => "",
        "php" => "",
        "swift" => "",
        "kt" | "kts" => "",
        "scala" => "",
        "lua" => "",
        "r" => "",
        
        // Web
        "html" | "htm" => "",
        "css" | "scss" | "sass" | "less" => "",
        "vue" => "",
        "svelte" => "",
        
        // Config
        "json" => "",
        "yaml" | "yml" => "",
        "toml" => "",
        "xml" => "",
        "ini" | "conf" | "config" => "",
        
        // Documents
        "md" | "markdown" => "",
        "txt" => "",
        "pdf" => "",
        "doc" | "docx" => "",
        "xls" | "xlsx" => "",
        "ppt" | "pptx" => "",
        
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "svg" => "",
        
        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "",
        
        // Video
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" => "",
        
        // Archives
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" => "",
        
        // Executables
        "exe" | "msi" | "app" | "dmg" => "",
        "sh" | "bash" | "zsh" | "fish" => "",
        
        // Other
        "lock" => "",
        "log" => "",
        "db" | "sqlite" | "sqlite3" => "",
        "gitignore" | "gitattributes" => "",
        "dockerfile" => "",
        
        _ => "",
    }
}

/// Check if a file is likely text/readable (for preview)
pub fn is_text_file(name: &str) -> bool {
    let text_extensions = [
        "txt", "md", "markdown", "rs", "py", "js", "ts", "jsx", "tsx",
        "html", "htm", "css", "scss", "sass", "less", "json", "yaml", "yml",
        "toml", "xml", "ini", "conf", "config", "sh", "bash", "zsh", "fish",
        "c", "h", "cpp", "hpp", "cc", "java", "kt", "go", "swift", "rb",
        "php", "lua", "r", "sql", "vim", "gitignore", "gitattributes",
        "dockerfile", "makefile", "cmake", "gradle", "properties", "env",
        "log", "csv", "tsv",
    ];
    
    let name_lower = name.to_lowercase();
    
    // Check extension
    if let Some(ext) = name_lower.rsplit('.').next() {
        if text_extensions.contains(&ext) {
            return true;
        }
    }
    
    // Check filename patterns
    let patterns = ["makefile", "dockerfile", "cmakelists", "readme", "license", "changelog"];
    for pattern in patterns {
        if name_lower.contains(pattern) {
            return true;
        }
    }
    
    false
}

/// Center a rectangle within another
pub fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::prelude::Rect) -> ratatui::prelude::Rect {
    use ratatui::prelude::*;
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a fixed-size centered rectangle
pub fn centered_rect_fixed(width: u16, height: u16, r: ratatui::prelude::Rect) -> ratatui::prelude::Rect {
    use ratatui::prelude::*;
    
    let width = width.min(r.width);
    let height = height.min(r.height);
    
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    
    Rect::new(x, y, width, height)
}
```
---
