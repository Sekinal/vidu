//! vidu - A blazingly fast disk usage analyzer
//!
//! A terminal-based disk usage analyzer with an interactive TUI,
//! parallel scanning, and caching support.
//!
//! # Usage
//!
//! ```bash
//! vidu [PATH]            # Analyze the given path (default: current directory)
//! vidu --help            # Show help
//! vidu --version         # Show version
//! vidu --fresh           # Force a fresh scan, ignoring cache
//! vidu --hidden          # Show hidden files
//! ```

mod analyzer;
mod app;
mod cache;
mod cleaner;
mod config;
mod constants;
mod error;
mod scanner;
mod ui;
mod utils;

use anyhow::{Context, Result};
use app::App;
use clap::Parser;
use config::{Config, SymbolModeConfig};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, panic, path::PathBuf, process::ExitCode};
use ui::terminal::TerminalCapabilities;
use ui::theme::ColorTheme;

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

    /// Clear the cache and exit
    #[arg(long)]
    clear_cache: bool,

    /// Hide hidden files (show all files by default)
    #[arg(long)]
    no_hidden: bool,

    /// Color theme (dracula, nord, gruvbox, catppuccin, solarized, tokyo-night, mono)
    #[arg(short, long)]
    theme: Option<String>,

    /// Symbol mode (auto, unicode, ascii)
    #[arg(long)]
    symbols: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {:#}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Handle --clear-cache before anything else
    if args.clear_cache {
        return clear_cache();
    }

    // Resolve and validate path
    let path = args
        .path
        .canonicalize()
        .with_context(|| format!("Cannot access path '{}'", args.path.display()))?;

    if !path.exists() {
        anyhow::bail!("Path '{}' does not exist", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("Path '{}' is not a directory", path.display());
    }

    // Load config from file
    let config = Config::load();

    // Detect terminal capabilities
    let caps = TerminalCapabilities::detect();

    // Determine theme (CLI > config > default)
    let theme_name = args.theme.as_deref().unwrap_or(&config.ui.theme);
    let theme = ColorTheme::by_name(theme_name).unwrap_or_default();
    ui::theme::init(theme);

    // Determine symbol mode (CLI > config > auto-detect)
    let symbol_mode = match args.symbols.as_deref() {
        Some("unicode") => ui::terminal::SymbolMode::Unicode,
        Some("ascii") => ui::terminal::SymbolMode::Ascii,
        Some(_) | None => match config.ui.symbols {
            SymbolModeConfig::Unicode => ui::terminal::SymbolMode::Unicode,
            SymbolModeConfig::Ascii => ui::terminal::SymbolMode::Ascii,
            SymbolModeConfig::Auto => caps.symbol_mode(),
        },
    };
    ui::symbols::init(symbol_mode);

    // Setup panic hook to restore terminal on panic
    setup_panic_hook();

    // Run the application with terminal setup/teardown
    // show_hidden = true by default, --no-hidden to hide
    run_with_terminal(path, args.fresh, !args.no_hidden)
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}

fn run_with_terminal(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<()> {
    // Setup terminal
    setup_terminal().context("Failed to setup terminal")?;

    // Run app and capture result
    let result = run_app(path, force_fresh, show_hidden);

    // Always restore terminal, even if app failed
    restore_terminal().context("Failed to restore terminal")?;

    result
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)
        .context("Failed to leave alternate screen")?;
    Ok(())
}

fn run_app(path: PathBuf, force_fresh: bool, show_hidden: bool) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let mut app = App::new(path, force_fresh, show_hidden).context("Failed to initialize app")?;

    app.run(&mut terminal).context("App error")?;

    Ok(())
}

fn clear_cache() -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vidu");

    if cache_dir.exists() {
        let entries = std::fs::read_dir(&cache_dir)?;
        let count = entries.count();
        std::fs::remove_dir_all(&cache_dir)?;
        println!("Cleared {} cached entries from {}", count, cache_dir.display());
    } else {
        println!("No cache found at {}", cache_dir.display());
    }

    Ok(())
}
