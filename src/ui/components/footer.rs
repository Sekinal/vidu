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
        AppState::DeleteConfirm | AppState::CleaningConfirm => {
            &[("y", "confirm"), ("n", "cancel")]
        }
        AppState::Preview => &[("↑↓", "scroll"), ("p/Esc", "close")],
        AppState::Help => &[("Esc", "close")],
        AppState::Search => &[("↵", "search"), ("Esc", "cancel")],
        AppState::Scanning => &[],
        // Analysis views
        AppState::JunkAnalysis
        | AppState::DuplicateAnalysis
        | AppState::FileTypeAnalysis
        | AppState::AgeAnalysis
        | AppState::LargeFilesView
        | AppState::CacheView => &[
            ("↑↓", "nav"),
            ("!", "mode"),
            ("c", "clean"),
            ("q", "close"),
        ],
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