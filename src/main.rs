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

// ==========================================
// DATA STRUCTURES & SCANNER
// ==========================================

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry {
    name: String,
    size: u64,
    path: PathBuf,
    is_dir: bool,
    children: Vec<Entry>,
    modified: Option<SystemTime>,
}

impl Entry {
    /// Scan a directory in parallel using Rayon
    fn scan(path: PathBuf) -> Self {
        let metadata = fs::symlink_metadata(&path).ok();
        let name = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("."))
            .to_string_lossy()
            .to_string();

        // Default file info
        let mut entry = Entry {
            name,
            size: 0,
            path: path.clone(),
            is_dir: false,
            children: vec![],
            modified: metadata.as_ref().and_then(|m| m.modified().ok()),
        };

        if let Some(meta) = metadata {
            if meta.is_dir() {
                entry.is_dir = true;
                // Collect entries first
                let entries: Vec<_> = fs::read_dir(&path)
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .collect();

                // Rayon parallel iteration for recursion
                entry.children = entries
                    .par_iter()
                    .map(|dir_entry| Entry::scan(dir_entry.path()))
                    .collect();
                
                // Sort by size descending
                entry.children.sort_by(|a, b| b.size.cmp(&a.size));
                
                // Sum size (including metadata size if needed, but usually just children)
                entry.size = entry.children.iter().map(|c| c.size).sum::<u64>() + meta.len();
            } else {
                entry.size = meta.len();
            }
        }

        entry
    }

    /// Update only the current node's children (Smart Refresh)
    fn refresh_children(&mut self) {
        if !self.is_dir { return; }
        
        let fresh_node = Entry::scan(self.path.clone());
        self.children = fresh_node.children;
        self.size = fresh_node.size;
        self.modified = fresh_node.modified;
    }
}

// ==========================================
// CACHING SYSTEM (UPDATED FOR BINCODE 2.0)
// ==========================================

struct CacheManager;

impl CacheManager {
    fn get_cache_path(scan_path: &std::path::Path) -> PathBuf {
        let dirs = ProjectDirs::from("com", "vidu", "vidu").unwrap();
        let cache_dir = dirs.cache_dir();
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(cache_dir);
        }
        
        // Create a unique hash for the path to avoid collisions
        let digest = md5::compute(scan_path.to_string_lossy().as_bytes());
        cache_dir.join(format!("{:x}.bin.lz4", digest))
    }

    fn save(root: &Entry) -> Result<()> {
        let path = Self::get_cache_path(&root.path);
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = lz4_flex::frame::FrameEncoder::new(writer);
        
        // BINCODE 2.0 CHANGE: Use bincode::serde::encode_into_std_write
        bincode::serde::encode_into_std_write(
            root,
            &mut encoder,
            bincode::config::standard()
        )?;
        
        encoder.finish()?;
        Ok(())
    }

    fn load(scan_path: &std::path::Path) -> Result<Entry> {
        let path = Self::get_cache_path(scan_path);
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut decoder = lz4_flex::frame::FrameDecoder::new(reader);
        
        // BINCODE 2.0 CHANGE: Use bincode::serde::decode_from_std_read
        let entry: Entry = bincode::serde::decode_from_std_read(
            &mut decoder,
            bincode::config::standard()
        )?;
        
        Ok(entry)
    }
}

// ==========================================
// UI & APP STATE
// ==========================================

struct App {
    root: Entry,
    // Stack of references to navigate into folders
    // We store indices of the children path
    nav_stack: Vec<usize>, 
    table_state: TableState,
    loading: bool,
    status_msg: String,
}

impl App {
    fn new(path: PathBuf) -> Self {
        // Try to load from cache first
        let (root, loaded_from_cache) = match CacheManager::load(&path) {
            Ok(entry) => (entry, true),
            Err(_) => (Entry {
                name: "Loading...".into(),
                size: 0,
                path: path.clone(),
                is_dir: true,
                children: vec![],
                modified: None,
            }, false),
        };

        let mut app = Self {
            root,
            nav_stack: vec![],
            table_state: TableState::default(),
            loading: !loaded_from_cache,
            status_msg: if loaded_from_cache { 
                "Cached. Press 'r' to refresh.".to_string() 
            } else { 
                "Scanning...".to_string() 
            },
        };
        
        app.table_state.select(Some(0));
        app
    }

    /// Returns the Entry currently being viewed
    fn current_view(&self) -> &Entry {
        let mut current = &self.root;
        for &idx in &self.nav_stack {
            if idx < current.children.len() {
                current = &current.children[idx];
            }
        }
        current
    }

    // We need a mutable reference to traverse and update
    fn get_current_view_mut(&mut self) -> &mut Entry {
        let mut current = &mut self.root;
        for &idx in &self.nav_stack {
             current = &mut current.children[idx];
        }
        current
    }

    fn up(&mut self) {
        let i = self.table_state.selected().unwrap_or(0);
        if i > 0 {
            self.table_state.select(Some(i - 1));
        }
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
            }
        }
    }

    fn go_back(&mut self) {
        if !self.nav_stack.is_empty() {
            let prev_selection = self.nav_stack.pop().unwrap();
            self.table_state.select(Some(prev_selection));
        }
    }

    fn refresh_current(&mut self) {
        self.loading = true;
        self.status_msg = "Refeshing current directory...".to_string();
        let target = self.get_current_view_mut();
        target.refresh_children();
        // Re-sort
        target.children.sort_by(|a, b| b.size.cmp(&a.size));
        
        self.loading = false;
        self.status_msg = "Refreshed.".to_string();
        let _ = CacheManager::save(&self.root);
    }
    
    fn full_rescan(&mut self) {
        self.loading = true;
        let path = self.root.path.clone();
        self.root = Entry::scan(path);
        self.nav_stack.clear();
        self.table_state.select(Some(0));
        self.loading = false;
        self.status_msg = "Full Scan Complete.".to_string();
        let _ = CacheManager::save(&self.root);
    }
}

// ==========================================
// MAIN LOOP
// ==========================================

fn main() -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Determine path
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let mut app = App::new(path.clone());

    if app.loading {
        terminal.draw(|f| {
            let layout = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .split(f.area());
            let block = Block::default().borders(Borders::ALL).title(" Vidu ");
            let para = Paragraph::new(format!("Scanning {}...", path.display()))
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(para, layout[0]);
        })?;
        
        app.full_rescan();
    }

    let res = run_app(&mut terminal, &mut app);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => app.down(),
                        KeyCode::Char('k') | KeyCode::Up => app.up(),
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.enter_dir(),
                        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => app.go_back(),
                        KeyCode::Char('r') => app.refresh_current(),
                        KeyCode::Char('R') => app.full_rescan(),
                        _ => {}
                    }
                }
            }
        }
    }
}

// ==========================================
// RENDERING
// ==========================================

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Table
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // 1. Header: Breadcrumbs / Path
    let current_entry = app.current_view();
    let path_text = current_entry.path.display().to_string();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" 📂 ", Style::default().fg(Color::Yellow)),
        Span::styled(path_text, Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    
    f.render_widget(header, chunks[0]);

    // 2. Main Table
    let parent_size = current_entry.size as f64;
    
    let rows: Vec<Row> = current_entry.children.iter().map(|item| {
        let is_dir = item.is_dir;
        let icon = if is_dir { "📁" } else { "📄" };
        let color = if is_dir { Color::Blue } else { Color::White };
        
        let size_str = format_size(item.size, DECIMAL);
        let percentage = if parent_size > 0.0 {
            (item.size as f64 / parent_size) * 100.0
        } else { 0.0 };

        // Create visual bar
        let bar_width = 20;
        let filled = (percentage / 100.0 * bar_width as f64) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        
        // Colorize the bar based on percentage
        let bar_style = if percentage > 50.0 {
            Style::default().fg(Color::Red)
        } else if percentage > 20.0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        Row::new(vec![
            Cell::from(format!("{} {}", icon, item.name)).style(Style::default().fg(color)),
            Cell::from(bar).style(bar_style),
            Cell::from(format!("{:.1}%", percentage)),
            Cell::from(size_str),
        ])
    }).collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Length(22),
        Constraint::Length(8),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Name", "Usage", "%", "Size"]).style(Style::default().fg(Color::Cyan)))
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT)) 
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(table, chunks[1], &mut app.table_state);

    // 3. Footer (Stats & Help)
    let help_text = "ESC/q: Quit | h/Left: Back | l/Right/Enter: Open | r: Refresh Dir | R: Rescan All";
    let footer = Paragraph::new(Line::from(vec![
        Span::raw(app.status_msg.as_str()),
        Span::raw(" | "),
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(footer, chunks[2]);
}