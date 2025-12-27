//! Popup dialogs (delete confirmation, etc.)

use crate::app::App;
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use crate::utils::{centered_rect, format_bytes};
use ratatui::{prelude::*, widgets::*};

/// Render delete confirmation popup (unified for single and multi-select)
pub fn render_delete_popup(f: &mut Frame, app: &App) {
    let t = theme();
    let syms = symbols();
    let area = centered_rect(70, 50, f.area());

    // Clear background
    f.render_widget(Clear, area);

    let count = app.pending_clean_items.len();
    let size = format_bytes(app.pending_clean_size);

    let mut text = vec![
        Line::from(""),
    ];

    if count == 1 {
        // Single item - show details
        if let Some((path, item_size)) = app.pending_clean_items.first() {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            let is_dir = path.is_dir();
            let item_type = if is_dir { "directory" } else { "file" };

            text.push(Line::from(vec![Span::raw(format!(
                "Delete this {}?",
                item_type
            ))]));
            text.push(Line::from(""));
            text.push(Line::from(vec![
                Span::styled("  Name: ", styles::dim()),
                Span::styled(name, styles::file()),
            ]));
            text.push(Line::from(vec![
                Span::styled("  Size: ", styles::dim()),
                Span::styled(format_bytes(*item_size), styles::size()),
            ]));
            text.push(Line::from(vec![
                Span::styled("  Path: ", styles::dim()),
                Span::raw(crate::utils::truncate_str(&path.to_string_lossy(), 50)),
            ]));
        }
    } else {
        // Multiple items
        text.push(Line::from(vec![Span::styled(
            format!("Delete {} items?", count),
            styles::warning(),
        )]));
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled("  Total size: ", styles::dim()),
            Span::styled(&size, styles::size()),
        ]));
        text.push(Line::from(""));

        // Show some paths (up to 5)
        text.push(Line::from(vec![Span::styled("  Items:", styles::dim())]));
        for (i, (path, _)) in app.pending_clean_items.iter().take(5).enumerate() {
            let display = crate::utils::truncate_str(&path.to_string_lossy(), 50);
            text.push(Line::from(format!("    {}. {}", i + 1, display)));
        }
        if count > 5 {
            text.push(Line::from(format!("    ... and {} more", count - 5)));
        }
    }

    text.extend(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("{} Permanent deletion cannot be undone!", syms.warning),
            styles::dim(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", styles::warning()),
            Span::raw(" = move to trash (safe)  "),
            Span::styled("Y", styles::danger()),
            Span::raw(" = delete permanently  "),
            Span::styled("Esc", styles::success()),
            Span::raw(" = cancel"),
        ]),
    ]);

    let popup = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.warning))
                .title(format!(" {} DELETE CONFIRMATION ", syms.warning))
                .title_style(styles::warning()),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(popup, area);
}

/// Render search input popup
pub fn render_search_popup(f: &mut Frame, app: &App) {
    let t = theme();
    let area = centered_rect(50, 15, f.area());

    f.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Search: ", styles::accent()),
            Span::styled(&app.search_query, Style::default().fg(t.fg)),
            Span::styled("▎", Style::default().fg(t.accent)), // Cursor
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
                .title(" Search ")
                .title_style(styles::accent()),
        )
        .alignment(Alignment::Center);

    f.render_widget(popup, area);
}