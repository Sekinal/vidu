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