//! Main table component for displaying entries

use crate::app::App;
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use crate::utils::{create_smooth_bar, format_age, format_bytes, get_file_icon, truncate_str};
use ratatui::{prelude::*, widgets::*};

/// Render the main file/directory table
pub fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    // We need to calculate rows first to avoid double borrowing app
    // (current_view borrows app, and we need &mut app for table_state later)
    let (rows, total_items, selected_idx) = {
        let current = app.current_view();
        let parent_size = current.size as f64;
        let marked_items = &app.marked_items;
        let show_hidden = app.show_hidden;

        // Filter out hidden files for display if show_hidden is false
        let visible_children: Vec<_> = current
            .children
            .iter()
            .filter(|item| show_hidden || !item.hidden)
            .collect();

        let syms = symbols();
        let t = theme();

        let rows: Vec<Row> = visible_children
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_marked = marked_items.contains(&item.path);
                let has_error = item.has_error();
                let icon = get_file_icon(&item.name, item.is_dir);

                // Junk indicator
                let junk_indicator = if let Some(jt) = item.junk_type {
                    format!(" {}", jt.icon())
                } else {
                    String::new()
                };

                // Name with icon - styled text
                let name_style = if is_marked {
                    styles::marked()
                } else if item.junk_type.is_some() {
                    styles::warning() // Highlight junk items
                } else if item.is_dir {
                    styles::directory()
                } else {
                    styles::file()
                };

                // Mark indicator using symbols
                let mark_indicator = if is_marked {
                    format!("{} ", syms.marked)
                } else {
                    format!("{} ", syms.unmarked)
                };
                let error_indicator = if has_error {
                    format!(" {}", syms.warning)
                } else {
                    String::new()
                };

                // Row background based on state
                let row_bg = if is_marked {
                    Some(t.bg_selection)
                } else if has_error {
                    Some(Color::Rgb(60, 30, 30)) // Subtle red tint
                } else if idx % 2 == 1 {
                    Some(t.bg_alt) // Alternating row
                } else {
                    None
                };
                
                // Calculate max name width (40% of area minus some padding)
                let max_name_width = (area.width as f64 * 0.35) as usize;
                let name_display = truncate_str(&item.name, max_name_width.saturating_sub(6));
                
                let name_cell = Cell::from(format!(
                    "{}{}  {}{}{}",
                    mark_indicator, icon, name_display, junk_indicator, error_indicator
                ))
                .style(name_style);
                
                // Size bar
                let percentage = if parent_size > 0.0 {
                    (item.size as f64 / parent_size) * 100.0
                } else {
                    0.0
                };

                let bar = create_smooth_bar(percentage, 12);
                let bar_color = t.bar_color(percentage);
                let bar_cell = Cell::from(bar).style(Style::default().fg(bar_color));

                // Percentage
                let pct_cell = Cell::from(format!("{:>5.1}%", percentage))
                    .style(Style::default().fg(bar_color));
                
                // Items count
                let items_cell = Cell::from(format!(
                    "{:>6}",
                    if item.is_dir {
                        item.file_count.to_string()
                    } else {
                        "-".to_string()
                    }
                ))
                .style(styles::dim());
                
                // Size
                let size_str = format_bytes(item.size);
                let size_cell = Cell::from(format!("{:>9}", size_str)).style(styles::size());
                
                // Age
                let age_str = format_age(item.modified);
                let age_cell = Cell::from(format!("{:>12}", age_str)).style(styles::dim());

                // Create row with optional background
                let mut row = Row::new(vec![
                    name_cell,
                    bar_cell,
                    pct_cell,
                    items_cell,
                    size_cell,
                    age_cell,
                ]);

                if let Some(bg) = row_bg {
                    row = row.style(Style::default().bg(bg));
                }

                row
            })
            .collect();

        (rows, visible_children.len(), app.table_state.selected().unwrap_or(0))
    };
    
    // Column widths
    let widths = [
        Constraint::Percentage(35),  // Name
        Constraint::Length(12),      // Bar
        Constraint::Length(7),       // Percent
        Constraint::Length(7),       // Items
        Constraint::Length(10),      // Size
        Constraint::Length(13),      // Age
    ];
    
    // Header
    let header = Row::new(vec![
        Cell::from(" Name"),
        Cell::from("Usage"),
        Cell::from("    %"),
        Cell::from(" Items"),
        Cell::from("     Size"),
        Cell::from("    Modified"),
    ])
    .style(styles::header())
    .height(1);

    let syms = symbols();

    // Empty state
    if rows.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("  {} This directory is empty", syms.info)),
            Line::from(""),
            Line::from(format!("  Press {} to go back", syms.arrow_left)),
        ])
        .style(styles::dim())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border()),
        );
        f.render_widget(empty_msg, area);
        return;
    }
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(styles::border()),
        )
        .row_highlight_style(styles::highlight())
        .highlight_symbol(format!("{} ", syms.selected));

    f.render_stateful_widget(table, area, &mut app.table_state);

    // Scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some(syms.arrow_up))
        .end_symbol(Some(syms.arrow_down));
    
    let mut scrollbar_state = ScrollbarState::new(total_items)
        .position(selected_idx);
    
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}