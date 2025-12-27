//! Header component with disk gauge and breadcrumbs

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::{format_bytes, truncate_str};
use ratatui::{prelude::*, widgets::*};

/// Render the disk usage gauge
pub fn render_disk_gauge(f: &mut Frame, app: &App, area: Rect) {
    let used = app.disk_total.saturating_sub(app.disk_available);
    let ratio = if app.disk_total > 0 {
        used as f64 / app.disk_total as f64
    } else {
        0.0
    };
    
    let color = Theme::bar_color(ratio * 100.0);
    
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border())
                .title(" 💾 Disk Usage ")
                .title_style(styles::accent()),
        )
        .gauge_style(Style::default().fg(color).bg(Theme::BG))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(format!(
            "{} / {} ({:.1}% used)",
            format_bytes(used),
            format_bytes(app.disk_total),
            ratio * 100.0
        ));
    
    f.render_widget(gauge, area);
}

/// Render the breadcrumb navigation path
pub fn render_breadcrumbs(f: &mut Frame, app: &App, area: Rect) {
    let current = app.current_view();
    let path_str = current.path.to_string_lossy();
    
    // Build breadcrumb spans
    let mut spans = vec![Span::styled(" 📂 ", styles::directory())];
    
    let max_width = area.width.saturating_sub(10) as usize;
    let display_path = truncate_str(&path_str, max_width);
    
    spans.push(Span::styled(
        display_path,
        Style::default().fg(Theme::FG).add_modifier(Modifier::BOLD),
    ));
    
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