use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use directories::ProjectDirs;
use humansize::{format_size, DECIMAL};
use ratatui::{prelude::*, widgets::*};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{
    fs,
    io::{self, BufReader, BufWriter},
    time::{Duration, SystemTime},
};
use sysinfo::Disks;

// ==========================================
// THEME & STYLING
// ==========================================
const COLOR_BG: Color = Color::Rgb(40, 42, 54); // Dracula Background
const COLOR_FG: Color = Color::Rgb(248, 248, 242);
const COLOR_SELECTION: Color = Color::Rgb(68, 71, 90);
const COLOR_ACCENT: Color = Color::Rgb(189, 147, 249); // Purple
const COLOR_DIR: Color = Color::Rgb(139, 233, 253); // Cyan
const COLOR_FILE: Color = Color::Rgb(255, 121, 198); // Pink
const COLOR_SIZE: Color = Color::Rgb(80, 250, 123); // Green
const COLOR_WARN: Color = Color::Rgb(255, 184, 108); // Orange
const COLOR_DANGER: Color = Color::Rgb(255, 85, 85); // Red

// ==========================================
// DATA STRUCTURES
// ==========================================

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry {
    name: String,
    size: u64,
    path: PathBuf,
    is_dir: bool,
    children: Vec<Entry>,
    modified: Option<SystemTime>,
    file_count: usize, // Track number of files for "Items" column
}

impl Entry {
    fn scan(path: PathBuf) -> Self {
        let metadata = fs::symlink_metadata(&path).ok();
        let name = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("."))
            .to_string_lossy()
            .to_string();

        let mut entry = Entry {
            name,
            size: 0,
            path: path.clone(),
            is_dir: false,
            children: vec![],
            modified: metadata.as_ref().and_then(|m| m.modified().ok()),
            file_count: 1,
        };

        if let Some(meta) = metadata {
            if meta.is_dir() {
                entry.is_dir = true;
                let entries: Vec<_> = fs::read_dir(&path)
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .collect();

                entry.children = entries
                    .par_iter()
                    .map(|dir_entry| Entry::scan(dir_entry.path()))
                    .collect();
                
                // Aggregate stats
                entry.size = entry.children.iter().map(|c| c.size).sum::<u64>() + meta.len();
                entry.file_count = entry.children.iter().map(|c| c.file_count).sum();
                
                // Initial Sort
                entry.children.sort_by(|a, b| b.size.cmp(&a.size));
            } else {
                entry.size = meta.len();
            }
        }

        entry
    }

    fn refresh_children(&mut self) {
        if !self.is_dir { return; }
        let fresh_node = Entry::scan(self.path.clone());
        self.children = fresh_node.children;
        self.size = fresh_node.size;
        self.file_count = fresh_node.file_count;
    }

    /// Recursively deletes the entry from disk
    fn delete_from_disk(&self) -> Result<()> {
        if self.is_dir {
            fs::remove_dir_all(&self.path)?;
        } else {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

// ==========================================
// CACHING SYSTEM
// ==========================================

struct CacheManager;

impl CacheManager {
    fn get_cache_path(scan_path: &std::path::Path) -> PathBuf {
        let dirs = ProjectDirs::from("com", "vidu", "vidu").unwrap();
        let cache_dir = dirs.cache_dir();
        if !cache_dir.exists() { let _ = fs::create_dir_all(cache_dir); }
        let digest = md5::compute(scan_path.to_string_lossy().as_bytes());
        cache_dir.join(format!("{:x}.bin.lz4", digest))
    }

    fn save(root: &Entry) -> Result<()> {
        let path = Self::get_cache_path(&root.path);
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = lz4_flex::frame::FrameEncoder::new(writer);
        bincode::serde::encode_into_std_write(root, &mut encoder, bincode::config::standard())?;
        encoder.finish()?;
        Ok(())
    }

    fn load(scan_path: &std::path::Path) -> Result<Entry> {
        let path = Self::get_cache_path(scan_path);
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut decoder = lz4_flex::frame::FrameDecoder::new(reader);
        let entry: Entry = bincode::serde::decode_from_std_read(&mut decoder, bincode::config::standard())?;
        Ok(entry)
    }
}

// ==========================================
// APPLICATION LOGIC
// ==========================================

#[derive(PartialEq, Clone, Copy)]
enum SortMode {
    Size,
    Name,
    Count,
}

#[derive(PartialEq)]
enum AppState {
    Browsing,
    DeleteConfirm,
}

struct App {
    root: Entry,
    nav_stack: Vec<usize>, 
    table_state: TableState,
    loading: bool,
    status_msg: String,
    // UX State
    sort_mode: SortMode,
    state: AppState,
    disk_info: (u64, u64), // Total, Available
}

impl App {
    fn new(path: PathBuf) -> Self {
        // Sysinfo 0.37+: Use Disks struct
        let disks = Disks::new_with_refreshed_list();
        
        // Find disk usage for the current path
        let mut disk_info = (0, 0);
        for disk in &disks {
            if path.starts_with(disk.mount_point()) {
                disk_info = (disk.total_space(), disk.available_space());
                break;
            }
        }

        let (root, loaded_from_cache) = match CacheManager::load(&path) {
            Ok(entry) => (entry, true),
            Err(_) => (Entry::scan(path.clone()), false), // Fallback to immediate scan if not cached
        };

        let mut app = Self {
            root,
            nav_stack: vec![],
            table_state: TableState::default(),
            loading: !loaded_from_cache,
            status_msg: if loaded_from_cache { "Cached.".into() } else { "Scanning...".into() },
            sort_mode: SortMode::Size,
            state: AppState::Browsing,
            disk_info,
        };
        
        app.table_state.select(Some(0));
        app
    }

    fn current_view(&self) -> &Entry {
        let mut current = &self.root;
        for &idx in &self.nav_stack {
            if idx < current.children.len() {
                current = &current.children[idx];
            }
        }
        current
    }

    fn get_current_view_mut(&mut self) -> &mut Entry {
        let mut current = &mut self.root;
        for &idx in &self.nav_stack {
             current = &mut current.children[idx];
        }
        current
    }

    fn sort_current(&mut self) {
        let mode = self.sort_mode; // Copy mode to avoid borrowing self in closure
        let target = self.get_current_view_mut();
        target.children.sort_by(|a, b| match mode {
            SortMode::Size => b.size.cmp(&a.size),
            SortMode::Count => b.file_count.cmp(&a.file_count),
            SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
    }

    fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Size => SortMode::Name,
            SortMode::Name => SortMode::Count,
            SortMode::Count => SortMode::Size,
        };
        self.sort_current();
        self.status_msg = format!("Sorted by {:?}", match self.sort_mode {
            SortMode::Size => "Size",
            SortMode::Name => "Name",
            SortMode::Count => "Item Count",
        });
    }

    fn up(&mut self) {
        let i = self.table_state.selected().unwrap_or(0);
        if i > 0 { self.table_state.select(Some(i - 1)); }
    }

    fn down(&mut self) {
        let i = self.table_state.selected().unwrap_or(0);
        let current = self.current_view();
        if i < current.children.len().saturating_sub(1) {
            self.table_state.select(Some(i + 1));
        }
    }

    fn enter_dir(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let current = self.current_view();
            if selected < current.children.len() && current.children[selected].is_dir {
                self.nav_stack.push(selected);
                self.table_state.select(Some(0));
                self.sort_current();
            }
        }
    }

    fn go_back(&mut self) {
        if !self.nav_stack.is_empty() {
            let prev_selection = self.nav_stack.pop().unwrap();
            self.table_state.select(Some(prev_selection));
        }
    }

    fn request_delete(&mut self) {
        if self.nav_stack.is_empty() && self.table_state.selected().is_none() { return; }
        self.state = AppState::DeleteConfirm;
    }

    fn confirm_delete(&mut self) {
        let selected_opt = self.table_state.selected();
        if selected_opt.is_none() {
            self.state = AppState::Browsing;
            return;
        }
        let selected = selected_opt.unwrap();

        // 1. Get info needed for deletion (Immutable Borrow)
        let (path, is_dir) = {
            let view = self.current_view();
            if selected >= view.children.len() {
                self.state = AppState::Browsing;
                return;
            }
            let item = &view.children[selected];
            (item.path.clone(), item.is_dir)
        };

        // 2. Perform Deletion (I/O) - No borrow on self
        let delete_res = if is_dir {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        // Handle I/O Error
        if let Err(e) = delete_res {
            self.status_msg = format!("Error deleting: {}", e);
            self.state = AppState::Browsing;
            return;
        }

        // 3. Update Memory Tree (Mutable Borrow)
        {
            let current_view = self.get_current_view_mut();
            // Verify index is still valid
            if selected < current_view.children.len() {
                let item = &current_view.children[selected];
                let deleted_size = item.size;
                let deleted_count = item.file_count;
                
                current_view.children.remove(selected);
                
                // Propagate size changes upwards (Imperfect but fast approach)
                // We subtract from the current view.
                if current_view.size >= deleted_size { current_view.size -= deleted_size; }
                if current_view.file_count >= deleted_count { current_view.file_count -= deleted_count; }

                // Adjust cursor
                let new_len = current_view.children.len();
                if selected >= new_len && new_len > 0 {
                    self.table_state.select(Some(new_len - 1));
                } else if new_len == 0 {
                    self.table_state.select(None);
                }
            }
        }

        self.status_msg = "Deleted. Cache updated.".to_string();
        let _ = CacheManager::save(&self.root);
        self.state = AppState::Browsing;
    }

    fn refresh_current(&mut self) {
        self.loading = true;
        self.status_msg = "Refreshing...".into();
        let target = self.get_current_view_mut();
        target.refresh_children();
        self.sort_current();
        self.loading = false;
        self.status_msg = "Refreshed.".into();
        let _ = CacheManager::save(&self.root);
    }
    
    fn full_rescan(&mut self) {
        self.loading = true;
        self.status_msg = "Scanning...".into();
        // In a real app, this scanning should be off the main thread to allow rendering the spinner
        self.root = Entry::scan(self.root.path.clone());
        self.nav_stack.clear();
        self.table_state.select(Some(0));
        self.loading = false;
        self.status_msg = "Done.".into();
        let _ = CacheManager::save(&self.root);
    }
}

// ==========================================
// MAIN LOOP
// ==========================================

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let path = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
    let mut app = App::new(path.clone());

    // Initial full scan if needed
    if app.loading && app.root.children.is_empty() {
         terminal.draw(|f| draw_loading(f, &path))?;
         app.full_rescan();
    }

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.state {
                        AppState::Browsing => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('j') | KeyCode::Down => app.down(),
                            KeyCode::Char('k') | KeyCode::Up => app.up(),
                            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.enter_dir(),
                            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => app.go_back(),
                            KeyCode::Char('d') => app.request_delete(),
                            KeyCode::Char('s') => app.toggle_sort(),
                            KeyCode::Char('r') => app.refresh_current(),
                            KeyCode::Char('R') => app.full_rescan(),
                            _ => {}
                        },
                        AppState::DeleteConfirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => app.confirm_delete(),
                            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => app.state = AppState::Browsing,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

// ==========================================
// UI RENDERERS
// ==========================================

fn draw_loading(f: &mut Frame, path: &PathBuf) {
    let area = f.area();
    let block = Block::default().borders(Borders::ALL).style(Style::default().fg(COLOR_ACCENT));
    let text = vec![
        Line::from(" INITIALIZING SCAN "),
        Line::from(path.to_string_lossy().to_string()),
        Line::from(" This may take a moment... "),
    ];
    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    
    // Center vertical
    let layout = Layout::default()
        .constraints([Constraint::Percentage(45), Constraint::Length(5), Constraint::Percentage(45)])
        .split(area);
    f.render_widget(p, layout[1]);
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();
    
    // Layout: Header (Disk Info) | Path | Table | Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Disk Bar
            Constraint::Length(3), // Breadcrumbs
            Constraint::Min(1),    // Table
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // 1. Disk Usage Bar
    let (total, avail) = app.disk_info;
    let used = total.saturating_sub(avail);
    let ratio = if total > 0 { used as f64 / total as f64 } else { 0.0 };
    
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_SELECTION)).title(" Physical Disk "))
        .gauge_style(Style::default().fg(COLOR_ACCENT).bg(COLOR_BG))
        .ratio(ratio)
        .label(format!("{} / {}", format_size(used, DECIMAL), format_size(total, DECIMAL)));
    f.render_widget(gauge, chunks[0]);

    // 2. Breadcrumbs
    let current_entry = app.current_view();
    let path_str = current_entry.path.to_string_lossy();
    
    let breadcrumb = Paragraph::new(Line::from(vec![
        Span::styled(" 📂 ", Style::default().fg(COLOR_DIR)),
        Span::styled(path_str, Style::default().fg(COLOR_FG).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_SELECTION)));
    f.render_widget(breadcrumb, chunks[1]);

    // 3. The Table
    render_table(f, app, chunks[2]);

    // 4. Footer
    let sort_str = match app.sort_mode {
        SortMode::Size => "Sort:Size",
        SortMode::Name => "Sort:Name",
        SortMode::Count => "Sort:Count",
    };
    
    let status_style = if app.loading { Style::default().fg(COLOR_WARN) } else { Style::default().fg(COLOR_ACCENT) };
    
    let keys = vec![
        ("ESC", "Back"), ("↵", "Open"), ("d", "Delete"), ("s", sort_str), ("r", "Refresh")
    ];
    
    let mut spans = vec![Span::styled(format!(" {} ", app.status_msg), status_style)];
    for (k, v) in keys {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(k, Style::default().fg(COLOR_DIR).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(format!(":{}", v)));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(COLOR_BG).fg(COLOR_FG));
    f.render_widget(footer, chunks[3]);

    // 5. Delete Popup Overlay
    if app.state == AppState::DeleteConfirm {
        render_delete_popup(f, app);
    }
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let current_entry = app.current_view();
    let parent_size = current_entry.size as f64;

    let rows: Vec<Row> = current_entry.children.iter().map(|item| {
        let is_dir = item.is_dir;
        let icon = if is_dir { "" } else { "" }; // Nerd Fonts (Fallback to Folder/File if needed)
        let color = if is_dir { COLOR_DIR } else { COLOR_FILE };
        
        let size_str = format_size(item.size, DECIMAL);
        let percentage = if parent_size > 0.0 { (item.size as f64 / parent_size) * 100.0 } else { 0.0 };
        
        // High-res visual bar
        let bar_width = 15;
        let bar = create_smooth_bar(percentage, bar_width);
        
        let bar_color = if percentage > 50.0 { COLOR_DANGER } else if percentage > 20.0 { COLOR_WARN } else { COLOR_SIZE };

        Row::new(vec![
            Cell::from(format!(" {}  {}", icon, item.name)).style(Style::default().fg(color)),
            Cell::from(bar).style(Style::default().fg(bar_color)),
            Cell::from(format!("{:.1}%", percentage)),
            Cell::from(item.file_count.to_string()),
            Cell::from(size_str).style(Style::default().fg(COLOR_FG)),
        ])
    }).collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Length(15),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Name", "Usage", "%", "Items", "Size"])
            .style(Style::default().fg(COLOR_SELECTION).bg(COLOR_DIR).add_modifier(Modifier::BOLD))
            .bottom_margin(0)
        )
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT).border_style(Style::default().fg(COLOR_SELECTION)))
        .row_highlight_style(Style::default().bg(COLOR_SELECTION));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_delete_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 25, f.area());
    
    // Clear the area behind popup
    f.render_widget(Clear, area); 

    if let Some(idx) = app.table_state.selected() {
        let item = &app.get_current_view_mut().children[idx];
        
        let block = Block::default().borders(Borders::ALL).style(Style::default().fg(COLOR_DANGER)).title(" ⚠ DELETE CONFIRMATION ");
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::raw("Are you sure you want to delete:")]),
            Line::from(vec![Span::styled(&item.name, Style::default().fg(COLOR_FILE).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::raw("This action cannot be undone!")]),
            Line::from(""),
            Line::from(vec![
                Span::styled("(y)", Style::default().fg(COLOR_DANGER).add_modifier(Modifier::BOLD)),
                Span::raw(" Yes, delete it."),
            ]),
            Line::from(vec![
                Span::styled("(n)", Style::default().fg(COLOR_SIZE).add_modifier(Modifier::BOLD)),
                Span::raw(" No, cancel."),
            ]),
        ];
        
        let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
        f.render_widget(p, area);
    }
}

// Helper: Create fractional bar "█▉▊▋▌▍▎▏"
fn create_smooth_bar(percent: f64, width: usize) -> String {
    let blocks = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];
    let total_filled = (percent / 100.0) * width as f64;
    let full_blocks = total_filled.floor() as usize;
    let remainder = total_filled - full_blocks as f64;
    let partial_idx = (remainder * 8.0).round() as usize;
    
    let mut s = String::new();
    for _ in 0..full_blocks { s.push_str(blocks[8]); }
    if full_blocks < width {
        s.push_str(blocks[partial_idx]);
        for _ in 0..(width - full_blocks - 1) { s.push(' '); }
    }
    s
}

// Helper: Center popup
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ]).split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ]).split(popup_layout[1])[1]
}