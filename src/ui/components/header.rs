//! Header component with disk gauge and breadcrumbs

use crate::app::App;
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use crate::utils::{format_bytes, truncate_str};
use ratatui::{prelude::*, widgets::*};

/// Render the disk usage gauge
pub fn render_disk_gauge(f: &mut Frame, app: &App, area: Rect) {
    let t = theme();
    let syms = symbols();

    let used = app.disk_total.saturating_sub(app.disk_available);
    let ratio = if app.disk_total > 0 {
        used as f64 / app.disk_total as f64
    } else {
        0.0
    };

    let percent = ratio * 100.0;
    let color = t.bar_color(percent);

    // Disk status badge
    let (status_icon, status_style) = if percent > 90.0 {
        (syms.disk_critical, styles::danger())
    } else if percent > 75.0 {
        (syms.disk_low, styles::warning())
    } else {
        (syms.disk_ok, styles::success())
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border())
                .title(format!(" {} Disk Usage {} ", status_icon,
                    if percent > 90.0 { "[CRITICAL]" }
                    else if percent > 75.0 { "[LOW]" }
                    else { "[OK]" }
                ))
                .title_style(status_style),
        )
        .gauge_style(Style::default().fg(color).bg(t.bg))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(format!(
            "{} / {} ({:.1}% used)",
            format_bytes(used),
            format_bytes(app.disk_total),
            percent
        ));

    f.render_widget(gauge, area);
}

/// Render the breadcrumb navigation path
pub fn render_breadcrumbs(f: &mut Frame, app: &App, area: Rect) {
    let t = theme();
    let syms = symbols();
    let current = app.current_view();
    let path_str = current.path.to_string_lossy();

    // Build breadcrumb spans with separators
    let mut spans = vec![Span::styled(
        format!(" {} ", syms.dir_closed),
        styles::directory(),
    )];

    let max_width = area.width.saturating_sub(30) as usize;

    // Create breadcrumb path with separators
    let parts: Vec<&str> = path_str.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        spans.push(Span::styled(
            "/",
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
        ));
    } else if parts.len() > 4 {
        // Truncate from the left for deep paths
        spans.push(Span::styled(
            format!("{}", syms.ellipsis),
            styles::dim(),
        ));
        for (i, part) in parts.iter().rev().take(3).rev().enumerate() {
            if i > 0 || parts.len() > 3 {
                spans.push(Span::styled(
                    syms.breadcrumb_sep.to_string(),
                    styles::dim(),
                ));
            }
            spans.push(Span::styled(
                truncate_str(part, max_width / 3),
                Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
            ));
        }
    } else {
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    syms.breadcrumb_sep.to_string(),
                    styles::dim(),
                ));
            }
            spans.push(Span::styled(
                truncate_str(part, max_width / parts.len().max(1)),
                Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
            ));
        }
    }

    // Add item count
    let count_str = format!(
        " │ {} items ({} files, {} dirs)",
        current.children.len(),
        current.file_count,
        current.dir_count
    );
    spans.push(Span::styled(count_str, styles::dim()));

    let breadcrumb = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles::border()),
    );

    f.render_widget(breadcrumb, area);
}