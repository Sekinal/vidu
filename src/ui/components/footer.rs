//! Footer component with status and key hints

use crate::app::{App, AppState};
use crate::config::DeletionMode;
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use crate::utils::format_bytes;
use ratatui::{prelude::*, widgets::*};

/// Render the footer with status and key hints
pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let t = theme();
    let syms = symbols();
    let mut spans = Vec::new();

    // Mode indicators (always visible on left)
    // Deletion mode badge
    let del_mode_style = match app.deletion_mode {
        DeletionMode::Trash => styles::success(),
        DeletionMode::Permanent => styles::danger(),
    };
    spans.push(Span::styled(
        format!(" [{}] ", app.deletion_mode.label().to_uppercase()),
        del_mode_style,
    ));

    // Hidden files indicator
    if app.show_hidden {
        spans.push(Span::styled("[.*] ", styles::dim()));
    }

    // Separator
    spans.push(Span::styled("│ ", styles::dim()));

    // Sort mode indicator
    let sort_indicator = format!(
        "Sort:{} {}",
        app.sort_mode.label(),
        match app.sort_order {
            crate::app::SortOrder::Ascending => syms.sort_asc,
            crate::app::SortOrder::Descending => syms.sort_desc,
        }
    );
    spans.push(Span::styled(sort_indicator, styles::dim()));

    // Marked items summary (if any)
    if !app.marked_items.is_empty() {
        let marked_size: u64 = app
            .marked_items
            .iter()
            .filter_map(|path| {
                app.current_view()
                    .children
                    .iter()
                    .find(|e| &e.path == path)
                    .map(|e| e.size)
            })
            .sum();

        spans.push(Span::styled(" │ ", styles::dim()));
        spans.push(Span::styled(
            format!("{} {} marked ({})", syms.marked, app.marked_items.len(), format_bytes(marked_size)),
            styles::marked(),
        ));
    }

    // Separator
    spans.push(Span::styled(" │ ", styles::dim()));

    // Status message (with appropriate color)
    let status_style = if app.error_msg.is_some() {
        styles::danger()
    } else if app.status_msg.contains("Scanning") || app.status_msg.contains("Refreshing") {
        styles::warning()
    } else {
        styles::accent()
    };

    let status_text = app.error_msg.as_ref().unwrap_or(&app.status_msg);
    spans.push(Span::styled(format!("{} ", status_text), status_style));

    // Key hints (context-aware, fewer hints)
    let keys: &[(&str, &str)] = match app.state {
        AppState::Browsing => &[
            ("?", "help"),
            ("q", "quit"),
        ],
        AppState::DeleteConfirm | AppState::CleaningConfirm => {
            &[("y", "confirm"), ("n", "cancel")]
        }
        AppState::Preview => &[("Esc", "close")],
        AppState::Help => &[("Esc", "close")],
        AppState::Search => &[("Enter", "search"), ("Esc", "cancel")],
        AppState::Scanning | AppState::Deleting => &[],
        AppState::JunkAnalysis
        | AppState::DuplicateAnalysis
        | AppState::FileTypeAnalysis
        | AppState::AgeAnalysis
        | AppState::LargeFilesView
        | AppState::CacheView => &[
            ("c", "clean"),
            ("Esc", "close"),
        ],
    };

    for (key, desc) in keys {
        spans.push(Span::styled(" │ ", styles::dim()));
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(t.info)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(":{}", desc), styles::dim()));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(t.bg));

    f.render_widget(footer, area);
}