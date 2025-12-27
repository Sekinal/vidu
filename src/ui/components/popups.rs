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