//! Popup dialogs (delete confirmation, etc.)

use crate::app::App;
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use crate::utils::{centered_rect, format_bytes};
use ratatui::{prelude::*, widgets::*};

/// Render delete confirmation popup
pub fn render_delete_popup(f: &mut Frame, app: &App) {
    let t = theme();
    let syms = symbols();
    let area = centered_rect(60, 40, f.area());

    // Clear background
    f.render_widget(Clear, area);

    // Get the selected item using visible-to-actual mapping
    let Some(item) = app.selected_item() else {
        return;
    };

    let item_type = if item.is_dir { "directory" } else { "file" };
    let warnings = if item.is_dir && item.file_count > 0 {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("{} This will delete {} files!", syms.warning, item.file_count),
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
            format!("{} This action cannot be undone!", syms.warning),
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
                .border_style(Style::default().fg(t.danger))
                .title(format!(" {} DELETE CONFIRMATION ", syms.warning))
                .title_style(styles::danger()),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(popup, area);
}

/// Render cleaning confirmation popup
pub fn render_cleaning_popup(f: &mut Frame, app: &App) {
    let t = theme();
    let syms = symbols();
    let area = centered_rect(70, 50, f.area());

    f.render_widget(Clear, area);

    let mode_str = match app.deletion_mode {
        crate::config::DeletionMode::Trash => "Move to Trash",
        crate::config::DeletionMode::Permanent => "PERMANENTLY DELETE",
    };

    let mode_style = match app.deletion_mode {
        crate::config::DeletionMode::Trash => styles::warning(),
        crate::config::DeletionMode::Permanent => styles::danger(),
    };

    let count = app.pending_clean_items.len();
    let size = format_bytes(app.pending_clean_size);

    let mut text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} {} items?", mode_str, count),
            mode_style,
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total size: ", styles::dim()),
            Span::styled(&size, styles::size()),
        ]),
        Line::from(""),
    ];

    // Show some paths (up to 5)
    text.push(Line::from(vec![Span::styled("  Items:", styles::dim())]));
    for (i, (path, _)) in app.pending_clean_items.iter().take(5).enumerate() {
        let display = crate::utils::truncate_str(&path.to_string_lossy(), 50);
        text.push(Line::from(format!("    {}. {}", i + 1, display)));
    }
    if count > 5 {
        text.push(Line::from(format!("    ... and {} more", count - 5)));
    }

    text.push(Line::from(""));

    // Warning based on mode
    if app.deletion_mode == crate::config::DeletionMode::Permanent {
        text.push(Line::from(vec![Span::styled(
            format!("  {} PERMANENT DELETE - Cannot be undone!", syms.warning),
            styles::danger(),
        )]));
    } else {
        text.push(Line::from(vec![Span::styled(
            format!("  {} Items will be moved to system trash", syms.trash),
            styles::dim(),
        )]));
    }

    text.extend(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Press "),
            Span::styled("y", if app.deletion_mode == crate::config::DeletionMode::Permanent { styles::danger() } else { styles::warning() }),
            Span::raw(" to confirm, "),
            Span::styled("n", styles::success()),
            Span::raw(" to cancel"),
        ]),
    ]);

    let title = match app.deletion_mode {
        crate::config::DeletionMode::Trash => format!(" {} CLEAN CONFIRMATION ", syms.trash),
        crate::config::DeletionMode::Permanent => format!(" {} PERMANENT DELETE CONFIRMATION ", syms.warning),
    };

    let popup = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(mode_style)
                .title(title)
                .title_style(mode_style),
        )
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