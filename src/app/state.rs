//! Application state management

use super::input::{
    Action, DeleteConfirmAction, HelpAction, KeyBindings, PreviewAction, SearchAction,
};
use crate::cache::CacheManager;
use crate::constants::ui::EVENT_POLL_TIMEOUT_MS;
use crate::scanner::{scan, Entry, ScanOptions, ScanProgress};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::*, widgets::TableState};
use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};
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
    Flat,
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

    pub fn label(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "Ascending",
            SortOrder::Descending => "Descending",
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
    pub marked_items: HashSet<PathBuf>,

    // Config
    pub force_fresh: bool,

    // Cache manager
    cache_manager: Option<CacheManager>,
}

impl App {
    pub fn new(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<Self> {
        // Get disk info
        let (disk_total, disk_available) = Self::get_disk_info(&path);

        // Initialize cache manager
        let cache_manager = CacheManager::new().ok();

        // Create scan options
        let scan_options = ScanOptions::default().with_hidden(show_hidden);

        // Try to load from cache unless force_fresh
        let (root, from_cache) = if !force_fresh {
            if let Some(ref cm) = cache_manager {
                match cm.load(&path) {
                    Ok(entry) => (entry, true),
                    Err(_) => (scan(path.clone(), &scan_options), false),
                }
            } else {
                (scan(path.clone(), &scan_options), false)
            }
        } else {
            (scan(path.clone(), &scan_options), false)
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
            marked_items: HashSet::new(),
            force_fresh,
            cache_manager,
        };

        // Select first item if available
        if !app.root.children.is_empty() {
            app.table_state.select(Some(0));
        }

        // Save cache if we just scanned
        if !from_cache {
            if let Some(ref cm) = app.cache_manager {
                let _ = cm.save(&app.root);
            }
        }

        Ok(app)
    }

    fn get_disk_info(path: &PathBuf) -> (u64, u64) {
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            if path.starts_with(disk.mount_point()) {
                return (disk.total_space(), disk.available_space());
            }
        }
        (0, 0)
    }

    /// Main event loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let poll_timeout = Duration::from_millis(EVENT_POLL_TIMEOUT_MS);

        loop {
            // Render
            terminal.draw(|f| ui::render::render(f, self))?;

            // Handle events
            if event::poll(poll_timeout)? {
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
    fn handle_key(
        &mut self,
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> bool {
        // Clear error message on any key
        self.error_msg = None;

        match self.state {
            AppState::Browsing => self.handle_browsing_key(code, modifiers),
            AppState::DeleteConfirm => self.handle_delete_confirm_key(code),
            AppState::Preview => self.handle_preview_key(code),
            AppState::Help => self.handle_help_key(code),
            AppState::Search => self.handle_search_key(code),
            AppState::Scanning => false,
        }
    }

    fn handle_browsing_key(
        &mut self,
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> bool {
        let action = KeyBindings::browsing_action(code, modifiers);

        match action {
            Action::Quit => return true,
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::GoToTop => self.go_to_top(),
            Action::GoToBottom => self.go_to_bottom(),
            Action::PageUp => self.page_up(),
            Action::PageDown => self.page_down(),
            Action::Enter => self.enter_dir(),
            Action::GoBack => {
                if self.nav_stack.is_empty() {
                    return true;
                }
                self.go_back();
            }
            Action::GoToRoot => self.go_to_root(),
            Action::Delete => self.request_delete(),
            Action::ToggleMark => self.toggle_mark(),
            Action::CycleSort => self.cycle_sort(),
            Action::ToggleSortOrder => self.toggle_sort_order(),
            Action::Refresh => self.refresh_current(),
            Action::FullRescan => self.full_rescan(),
            Action::TogglePreview => self.toggle_preview(),
            Action::ShowHelp => self.state = AppState::Help,
            Action::StartSearch => {
                self.search_query.clear();
                self.state = AppState::Search;
            }
            Action::ToggleHidden => self.toggle_hidden(),
            Action::None => {}
        }

        false
    }

    fn handle_delete_confirm_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        match KeyBindings::delete_confirm_action(code) {
            DeleteConfirmAction::Confirm => self.confirm_delete(),
            DeleteConfirmAction::Cancel => self.state = AppState::Browsing,
            DeleteConfirmAction::None => {}
        }
        false
    }

    fn handle_preview_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        match KeyBindings::preview_action(code) {
            PreviewAction::Close => self.state = AppState::Browsing,
            PreviewAction::ScrollUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(1);
            }
            PreviewAction::ScrollDown => {
                if self.preview_scroll < self.preview_content.len().saturating_sub(1) {
                    self.preview_scroll += 1;
                }
            }
            PreviewAction::PageUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(20);
            }
            PreviewAction::PageDown => {
                self.preview_scroll = (self.preview_scroll + 20)
                    .min(self.preview_content.len().saturating_sub(1));
            }
            PreviewAction::GoToTop => self.preview_scroll = 0,
            PreviewAction::GoToBottom => {
                self.preview_scroll = self.preview_content.len().saturating_sub(1);
            }
            PreviewAction::None => {}
        }
        false
    }

    fn handle_help_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        if matches!(KeyBindings::help_action(code), HelpAction::Close) {
            self.state = AppState::Browsing;
        }
        false
    }

    fn handle_search_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        match KeyBindings::search_action(code) {
            SearchAction::Cancel => {
                self.state = AppState::Browsing;
                self.search_query.clear();
            }
            SearchAction::Execute => {
                self.execute_search();
                self.state = AppState::Browsing;
            }
            SearchAction::AddChar(c) => self.search_query.push(c),
            SearchAction::Backspace => {
                self.search_query.pop();
            }
            SearchAction::None => {}
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
        self.table_state
            .selected()
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
        crate::constants::ui::DEFAULT_VISIBLE_ROWS
    }

    /// Save current state to cache
    pub fn save_to_cache(&self) {
        if let Some(ref cm) = self.cache_manager {
            let _ = cm.save(&self.root);
        }
    }
}
