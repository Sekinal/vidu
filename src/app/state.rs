//! Application state management

use super::input::{
    Action, DeleteConfirmAction, HelpAction, KeyBindings, PreviewAction, SearchAction,
};
use crate::analyzer::{
    AgeAnalysis, CacheDetector, CacheLocation, DuplicateFinder, DuplicateResult,
    FileTypeAnalysis, LargeFile, LargeFileFinder,
};
use crate::cache::CacheManager;
use crate::config::DeletionMode;
use crate::constants::ui::EVENT_POLL_TIMEOUT_MS;
use crate::scanner::{scan_with_progress, Entry, JunkType, ScanOptions, ScanProgress};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::TableState};
use std::{collections::HashSet, path::PathBuf, sync::Arc, thread, time::Duration};
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
    // Cleaning utility states
    JunkAnalysis,
    DuplicateAnalysis,
    FileTypeAnalysis,
    AgeAnalysis,
    LargeFilesView,
    CacheView,
    CleaningConfirm,
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

    // Cleaning utility state
    pub deletion_mode: DeletionMode,

    // Analysis results (lazy-computed)
    pub junk_stats: Option<(usize, u64)>,           // (count, size)
    pub duplicate_result: Option<DuplicateResult>,
    pub file_type_analysis: Option<FileTypeAnalysis>,
    pub age_analysis: Option<AgeAnalysis>,
    pub large_files: Option<Vec<LargeFile>>,
    pub system_caches: Option<Vec<CacheLocation>>,

    // Analysis view state
    pub analysis_scroll: usize,
    pub analysis_selected: usize,

    // Cleaning state
    pub pending_clean_paths: Vec<PathBuf>,
    pub pending_clean_size: u64,
    pub last_clean_result: Option<crate::cleaner::CleaningResult>,
}

impl App {
    /// Create a new app - this will be in Scanning state initially
    pub fn new(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<Self> {
        // Get disk info
        let (disk_total, disk_available) = Self::get_disk_info(&path);

        // Initialize cache manager
        let cache_manager = CacheManager::new().ok();

        // Try to load from cache first (instant)
        let cached_entry = if !force_fresh {
            cache_manager.as_ref().and_then(|cm| cm.load(&path).ok())
        } else {
            None
        };

        let (root, state, status_msg) = if let Some(entry) = cached_entry {
            (entry, AppState::Browsing, "Loaded from cache".to_string())
        } else {
            // Create empty root - will scan with progress
            let name = path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string();
            let mut root = Entry::new(path.clone(), name);
            root.is_dir = true;
            (root, AppState::Scanning, "Scanning...".to_string())
        };

        let mut app = Self {
            root,
            original_path: path,
            nav_stack: Vec::new(),
            table_state: TableState::default(),
            state,
            view_mode: ViewMode::Tree,
            sort_mode: SortMode::Size,
            sort_order: SortOrder::Descending,
            show_hidden,
            status_msg,
            error_msg: None,
            disk_total,
            disk_available,
            preview_content: Vec::new(),
            preview_scroll: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            scan_progress: if state == AppState::Scanning {
                Some(ScanProgress::new())
            } else {
                None
            },
            marked_items: HashSet::new(),
            force_fresh,
            cache_manager,
            deletion_mode: DeletionMode::Trash,
            junk_stats: None,
            duplicate_result: None,
            file_type_analysis: None,
            age_analysis: None,
            large_files: None,
            system_caches: None,
            analysis_scroll: 0,
            analysis_selected: 0,
            pending_clean_paths: Vec::new(),
            pending_clean_size: 0,
            last_clean_result: None,
        };

        // Select first item if available (for cached entries)
        if !app.root.children.is_empty() {
            app.table_state.select(Some(0));
        }

        Ok(app)
    }

    /// Start the background scan (call this after showing initial UI)
    fn start_background_scan(&mut self) {
        if self.state != AppState::Scanning {
            return;
        }

        let path = self.original_path.clone();
        let show_hidden = self.show_hidden;
        let progress = self.scan_progress.clone();

        // Spawn scanning thread
        let scan_options = ScanOptions::default().with_hidden(show_hidden);

        // We need to do the scan in the main thread for now since Entry isn't Send
        // But we can still show progress during iteration
        if let Some(ref p) = progress {
            self.root = scan_with_progress(path, &scan_options, Some(p.clone()));
            p.mark_complete();
        }

        // Transition to browsing
        self.state = AppState::Browsing;
        self.scan_progress = None;

        if !self.root.children.is_empty() {
            self.table_state.select(Some(0));
        }

        // Update status
        self.status_msg = format!(
            "Scanned {} files, {} dirs",
            self.root.file_count, self.root.dir_count
        );

        // Save to cache
        self.save_to_cache();
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
        // If we need to scan, do it with live progress updates
        if self.state == AppState::Scanning {
            self.run_with_scanning(terminal)?;
        }

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

    /// Run the scanning phase with live progress updates
    fn run_with_scanning<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let path = self.original_path.clone();
        let show_hidden = self.show_hidden;
        let progress = self.scan_progress.clone().unwrap();

        // Spawn the scan in a background thread
        let scan_progress = progress.clone();
        let scan_path = path.clone();
        let scan_handle = thread::spawn(move || {
            let scan_options = ScanOptions::default().with_hidden(show_hidden);
            let result = scan_with_progress(scan_path, &scan_options, Some(scan_progress.clone()));
            scan_progress.mark_complete();
            result
        });

        // Show progress while scanning
        let poll_timeout = Duration::from_millis(50); // Fast updates for smooth progress

        while !progress.is_complete() {
            // Render progress
            terminal.draw(|f| ui::render::render(f, self))?;

            // Check for quit key
            if event::poll(poll_timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                            progress.cancel();
                            break;
                        }
                    }
                }
            }
        }

        // Get the scan result
        match scan_handle.join() {
            Ok(root) => {
                self.root = root;
                self.state = AppState::Browsing;
                self.scan_progress = None;

                if !self.root.children.is_empty() {
                    self.table_state.select(Some(0));
                }

                self.status_msg = format!(
                    "Scanned {} files, {} dirs",
                    self.root.file_count, self.root.dir_count
                );

                // Save to cache
                self.save_to_cache();
            }
            Err(_) => {
                self.error_msg = Some("Scan failed".to_string());
                self.state = AppState::Browsing;
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
            AppState::Scanning => {
                // Allow quit during scanning
                matches!(code, KeyCode::Char('q') | KeyCode::Esc)
            }
            // Analysis views share common input handling
            AppState::JunkAnalysis
            | AppState::DuplicateAnalysis
            | AppState::FileTypeAnalysis
            | AppState::AgeAnalysis
            | AppState::LargeFilesView
            | AppState::CacheView => self.handle_analysis_key(code),
            AppState::CleaningConfirm => self.handle_cleaning_confirm_key(code),
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
            // Cleaning utility actions
            Action::ShowJunkAnalysis => self.show_junk_analysis(),
            Action::ShowDuplicates => self.show_duplicates(),
            Action::ShowFileTypes => self.show_file_types(),
            Action::ShowOldFiles => self.show_old_files(),
            Action::ShowLargeFiles => self.show_large_files(),
            Action::ShowCaches => self.show_caches(),
            Action::CleanSelected => self.prepare_clean_marked(),
            Action::CleanAllJunk => self.prepare_clean_all_junk(),
            Action::ToggleDeletionMode => self.toggle_deletion_mode(),
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

    fn handle_cleaning_confirm_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        match KeyBindings::delete_confirm_action(code) {
            DeleteConfirmAction::Confirm => self.execute_cleaning(),
            DeleteConfirmAction::Cancel => self.cancel_cleaning(),
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

    fn handle_analysis_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        use super::input::AnalysisAction;

        // Get the max items for current view
        let max_items = match self.state {
            AppState::DuplicateAnalysis => {
                self.duplicate_result.as_ref().map(|r| r.groups.len()).unwrap_or(0)
            }
            AppState::LargeFilesView => {
                self.large_files.as_ref().map(|f| f.len()).unwrap_or(0)
            }
            AppState::CacheView => {
                self.system_caches.as_ref().map(|c| c.len()).unwrap_or(0)
            }
            _ => 0,
        };

        match KeyBindings::analysis_action(code) {
            AnalysisAction::Close => self.close_analysis(),
            AnalysisAction::ScrollUp => self.analysis_scroll_up(),
            AnalysisAction::ScrollDown => self.analysis_scroll_down(max_items),
            AnalysisAction::PageUp => self.analysis_page_up(),
            AnalysisAction::PageDown => self.analysis_page_down(max_items),
            AnalysisAction::GoToTop => self.analysis_go_to_top(),
            AnalysisAction::GoToBottom => self.analysis_go_to_bottom(max_items),
            AnalysisAction::ToggleDeletionMode => self.toggle_deletion_mode(),
            AnalysisAction::Select | AnalysisAction::ToggleMark => {
                // Mark selected item for cleaning
                self.clean_analysis_selected();
            }
            AnalysisAction::CleanSelected => self.clean_analysis_selected(),
            AnalysisAction::CleanAll => self.prepare_clean_all_junk(),
            AnalysisAction::None => {}
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

    /// Get visible children (respects show_hidden setting)
    pub fn visible_children(&self) -> Vec<&Entry> {
        self.current_view()
            .children
            .iter()
            .filter(|e| self.show_hidden || !e.hidden)
            .collect()
    }

    /// Get visible children count
    pub fn visible_children_count(&self) -> usize {
        self.current_view()
            .children
            .iter()
            .filter(|e| self.show_hidden || !e.hidden)
            .count()
    }

    /// Map visible index to actual index in children array
    pub fn visible_to_actual_index(&self, visible_idx: usize) -> Option<usize> {
        let children = &self.current_view().children;
        let mut visible_count = 0;
        for (actual_idx, child) in children.iter().enumerate() {
            if self.show_hidden || !child.hidden {
                if visible_count == visible_idx {
                    return Some(actual_idx);
                }
                visible_count += 1;
            }
        }
        None
    }

    /// Map actual index to visible index in children array
    pub fn actual_to_visible_index(&self, actual_idx: usize) -> Option<usize> {
        let children = &self.current_view().children;
        if actual_idx >= children.len() {
            return None;
        }
        // If the item at actual_idx is hidden and we're not showing hidden, return None
        if !self.show_hidden && children[actual_idx].hidden {
            return None;
        }
        // Count visible items before this index
        let visible_idx = children[..actual_idx]
            .iter()
            .filter(|c| self.show_hidden || !c.hidden)
            .count();
        Some(visible_idx)
    }

    /// Get selected item in current view (respects show_hidden)
    pub fn selected_item(&self) -> Option<&Entry> {
        let visible_idx = self.table_state.selected()?;
        let actual_idx = self.visible_to_actual_index(visible_idx)?;
        self.current_view().children.get(actual_idx)
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

    // ===== Cleaning Utility Methods =====

    /// Toggle deletion mode between Trash and Permanent
    pub fn toggle_deletion_mode(&mut self) {
        self.deletion_mode = self.deletion_mode.toggle();
        self.status_msg = format!("Deletion mode: {}", self.deletion_mode.label());
    }

    /// Compute junk statistics (lazy)
    pub fn compute_junk_stats(&mut self) {
        if self.junk_stats.is_none() {
            let stats = self.root.junk_stats();
            self.junk_stats = Some(stats);
        }
    }

    /// Compute duplicate detection (lazy)
    pub fn compute_duplicates(&mut self) {
        if self.duplicate_result.is_none() {
            let finder = DuplicateFinder::new();
            self.duplicate_result = Some(finder.find_duplicates(&self.root));
        }
    }

    /// Compute file type analysis (lazy)
    pub fn compute_file_types(&mut self) {
        if self.file_type_analysis.is_none() {
            self.file_type_analysis = Some(FileTypeAnalysis::from_entry(&self.root));
        }
    }

    /// Compute age analysis (lazy)
    pub fn compute_age_analysis(&mut self) {
        if self.age_analysis.is_none() {
            self.age_analysis = Some(AgeAnalysis::from_entry(&self.root));
        }
    }

    /// Compute large files list (lazy)
    pub fn compute_large_files(&mut self) {
        if self.large_files.is_none() {
            let mut finder = LargeFileFinder::new(100);
            finder.process_tree(&self.root);
            self.large_files = Some(finder.get_files());
        }
    }

    /// Detect system caches (lazy)
    pub fn detect_system_caches(&mut self) {
        if self.system_caches.is_none() {
            self.system_caches = CacheDetector::new().map(|d| d.detect_all());
        }
    }

    /// Get junk entries from current view
    pub fn collect_junk_entries(&self) -> Vec<&Entry> {
        self.root.collect_junk()
    }

    /// Show junk analysis view
    pub fn show_junk_analysis(&mut self) {
        self.compute_junk_stats();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::JunkAnalysis;
    }

    /// Show duplicates view
    pub fn show_duplicates(&mut self) {
        self.compute_duplicates();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::DuplicateAnalysis;
    }

    /// Show file types view
    pub fn show_file_types(&mut self) {
        self.compute_file_types();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::FileTypeAnalysis;
    }

    /// Show age analysis view
    pub fn show_old_files(&mut self) {
        self.compute_age_analysis();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::AgeAnalysis;
    }

    /// Show large files view
    pub fn show_large_files(&mut self) {
        self.compute_large_files();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::LargeFilesView;
    }

    /// Show system caches view
    pub fn show_caches(&mut self) {
        self.detect_system_caches();
        self.analysis_scroll = 0;
        self.analysis_selected = 0;
        self.state = AppState::CacheView;
    }

    /// Clear all cached analysis results (call after scan or refresh)
    pub fn invalidate_analysis(&mut self) {
        self.junk_stats = None;
        self.duplicate_result = None;
        self.file_type_analysis = None;
        self.age_analysis = None;
        self.large_files = None;
        // Don't invalidate system_caches as they're system-wide
    }
}
