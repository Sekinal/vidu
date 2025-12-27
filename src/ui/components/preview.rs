//! File preview component

use crate::app::App;
use crate::ui::theme::{styles, Theme};
use crate::utils::centered_rect;
use ratatui::{prelude::*, widgets::*};

/// Render file preview panel
pub fn render_preview(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());
    
    f.render_widget(Clear, area);
    
    let title = if let Some(item) = app.selected_item() {
        format!(" Preview: {} ", item.name)
    } else {
        " Preview ".to_string()
    };
    
    // Calculate visible lines
    let inner_height = area.height.saturating_sub(2) as usize;
    let total_lines = app.preview_content.len();
    let start = app.preview_scroll;
    let end = (start + inner_height).min(total_lines);
    
    let visible_lines: Vec<Line> = app.preview_content[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            // Line numbers
            let line_num = format!("{:>4} │ ", start + i + 1);
            let mut spans = vec![Span::styled(line_num, styles::dim())];
            
            // Syntax highlighting (basic)
            spans.push(Span::styled(line, Style::default().fg(Theme::FG)));
            
            Line::from(spans)
        })
        .collect();
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::accent())
        .title(title)
        .title_style(styles::accent());
    
    let paragraph = Paragraph::new(visible_lines).block(block);
    
    f.render_widget(paragraph, area);
    
    // Scroll indicator
    if total_lines > inner_height {
        let scroll_info = format!(
            " Lines {}-{} of {} ({}%) ",
            start + 1,
            end,
            total_lines,
            ((start as f64 / total_lines.saturating_sub(inner_height) as f64) * 100.0) as u32
        );
        
        let info_area = Rect {
            x: area.x + area.width.saturating_sub(scroll_info.len() as u16 + 2),
            y: area.y + area.height - 1,
            width: scroll_info.len() as u16,
            height: 1,
        };
        
        let info = Paragraph::new(scroll_info).style(styles::dim());
        f.render_widget(info, info_area);
    }
}