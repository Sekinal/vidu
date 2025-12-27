//! Main render function

use crate::app::{App, AppState};
use crate::ui::components::{footer, header, help, popups, preview, table};
use crate::ui::theme::{styles, Theme};
use ratatui::{prelude::*, widgets::*};

/// Main render function
pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    
    // Clear with background color
    f.render_widget(
        Block::default().style(Style::default().bg(Theme::BG)),
        area,
    );
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Disk gauge
            Constraint::Length(3),  // Breadcrumbs
            Constraint::Min(5),     // Table
            Constraint::Length(1),  // Footer
        ])
        .split(area);
    
    // Render main components
    header::render_disk_gauge(f, app, chunks[0]);
    header::render_breadcrumbs(f, app, chunks[1]);
    table::render_table(f, app, chunks[2]);
    footer::render_footer(f, app, chunks[3]);
    
    // Render overlays based on state
    match app.state {
        AppState::DeleteConfirm => {
            popups::render_delete_popup(f, app);
        }
        AppState::Preview => {
            preview::render_preview(f, app);
        }
        AppState::Help => {
            help::render_help(f, app);
        }
        AppState::Search => {
            popups::render_search_popup(f, app);
        }
        AppState::Scanning => {
            render_scanning_overlay(f, app);
        }
        AppState::Browsing => {}
    }
}

/// Render scanning progress overlay
fn render_scanning_overlay(f: &mut Frame, app: &App) {
    let area = crate::utils::centered_rect(50, 25, f.area());
    
    f.render_widget(Clear, area);
    
    let progress_text = if let Some(ref progress) = app.scan_progress {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(" Scanning... ", styles::accent())]),
            Line::from(""),
            Line::from(vec![Span::raw(format!(
                " Files: {} | Dirs: {}",
                progress.files(),
                progress.dirs()
            ))]),
            Line::from(vec![Span::raw(format!(
                " Bytes: {}",
                crate::utils::format_bytes(progress.bytes())
            ))]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Current: ",
                styles::dim(),
            )]),
            Line::from(vec![Span::raw(crate::utils::truncate_str(
                &progress.current(),
                40,
            ))]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(" Scanning... ", styles::accent())]),
            Line::from(""),
            Line::from(vec![Span::raw(" Please wait...")]),
        ]
    };
    
    let popup = Paragraph::new(progress_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::accent())
                .title(" 🔄 Scanning ")
                .title_style(styles::accent()),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(popup, area);
}