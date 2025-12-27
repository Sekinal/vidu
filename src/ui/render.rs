//! Main render function

use crate::app::{App, AppState};
use crate::ui::components::{footer, header, help, popups, preview, table};
use crate::ui::symbols::symbols;
use crate::ui::theme::{styles, theme};
use ratatui::{prelude::*, widgets::*};

/// Main render function
pub fn render(f: &mut Frame, app: &mut App) {
    let t = theme();
    let area = f.area();

    // Clear with background color
    f.render_widget(
        Block::default().style(Style::default().bg(t.bg)),
        area,
    );

    // If scanning, just show the scanning overlay (full screen)
    if app.state == AppState::Scanning {
        render_scanning_overlay(f, app, area);
        return;
    }

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
            // Already handled above
        }
        // Analysis views - rendered as overlays
        AppState::JunkAnalysis => {
            render_analysis_overlay(f, app, "Junk Analysis", junk_analysis_content(app));
        }
        AppState::DuplicateAnalysis => {
            render_analysis_overlay(f, app, "Duplicate Files", duplicate_analysis_content(app));
        }
        AppState::FileTypeAnalysis => {
            render_analysis_overlay(f, app, "File Types", file_type_analysis_content(app));
        }
        AppState::AgeAnalysis => {
            render_analysis_overlay(f, app, "Old Files", age_analysis_content(app));
        }
        AppState::LargeFilesView => {
            render_analysis_overlay(f, app, "Large Files", large_files_content(app));
        }
        AppState::CacheView => {
            render_analysis_overlay(f, app, "System Caches", cache_view_content(app));
        }
        AppState::CleaningConfirm => {
            popups::render_cleaning_popup(f, app);
        }
        AppState::Browsing => {}
    }
}

/// Render scanning progress overlay with spinner (full screen)
fn render_scanning_overlay(f: &mut Frame, app: &App, full_area: Rect) {
    let syms = symbols();
    // Center the content in the full area
    let area = crate::utils::centered_rect(50, 40, full_area);

    // Get spinner frame based on time (using file count as proxy for animation)
    let spinner_frame = if let Some(ref progress) = app.scan_progress {
        progress.files() / 10 // Changes every 10 files
    } else {
        0
    };
    let spinner = syms.spinner_frame(spinner_frame);

    let progress_text = if let Some(ref progress) = app.scan_progress {
        let files = progress.files();
        let dirs = progress.dirs();
        let bytes = progress.bytes();

        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" {} ", spinner), styles::accent()),
                Span::styled("Scanning...", styles::accent()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Files: ", styles::dim()),
                Span::styled(format!("{:>8}", files), styles::size()),
                Span::raw("   "),
                Span::styled("Dirs: ", styles::dim()),
                Span::styled(format!("{:>6}", dirs), styles::size()),
            ]),
            Line::from(vec![
                Span::styled("  Size:  ", styles::dim()),
                Span::styled(
                    format!("{:>8}", crate::utils::format_bytes(bytes)),
                    styles::size(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(" Current:", styles::dim())]),
            Line::from(vec![Span::raw(format!(
                "  {}",
                crate::utils::truncate_str(&progress.current(), 40)
            ))]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(" Press {} to cancel", syms.cross),
                styles::dim(),
            )]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" {} ", spinner), styles::accent()),
                Span::styled("Scanning...", styles::accent()),
            ]),
            Line::from(""),
            Line::from(vec![Span::raw(" Please wait...")]),
        ]
    };

    let popup = Paragraph::new(progress_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::accent())
                .title(format!(" {} Scanning ", syms.refresh))
                .title_style(styles::accent()),
        )
        .alignment(Alignment::Center);

    f.render_widget(popup, area);
}

/// Render analysis overlay
fn render_analysis_overlay(f: &mut Frame, app: &App, title: &str, content: Vec<Line<'static>>) {
    let area = crate::utils::centered_rect(80, 80, f.area());

    f.render_widget(Clear, area);

    let mode_indicator = match app.deletion_mode {
        crate::config::DeletionMode::Trash => " [Trash]",
        crate::config::DeletionMode::Permanent => " [PERMANENT]",
    };

    let popup = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::accent())
                .title(format!(" {} {} ", title, mode_indicator))
                .title_style(styles::accent())
                .title_bottom(" q: Close | !: Toggle Mode | c: Clean Selected ")
                .title_alignment(Alignment::Center),
        )
        .scroll((app.analysis_scroll as u16, 0));

    f.render_widget(popup, area);
}

/// Generate junk analysis content
fn junk_analysis_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some((count, size)) = app.junk_stats {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Total Junk: ", styles::accent()),
            Span::raw(format!("{} items, {}", count, crate::utils::format_bytes(size))),
        ]));
        lines.push(Line::from(""));

        // Group by junk type
        let junk_entries = app.root.collect_junk();
        let mut by_type: std::collections::HashMap<crate::scanner::JunkType, (usize, u64)> =
            std::collections::HashMap::new();

        for entry in &junk_entries {
            if let Some(jt) = entry.junk_type {
                let e = by_type.entry(jt).or_default();
                e.0 += 1;
                e.1 += entry.size;
            }
        }

        for (jt, (c, s)) in by_type.iter() {
            lines.push(Line::from(format!(
                "   {} {}: {} items, {}",
                jt.icon(),
                jt.label(),
                c,
                crate::utils::format_bytes(*s)
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No junk detected"));
    }

    lines
}

/// Generate duplicate analysis content
fn duplicate_analysis_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref result) = app.duplicate_result {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Duplicates Found: ", styles::accent()),
            Span::raw(format!(
                "{} groups, {} wasted",
                result.groups.len(),
                crate::utils::format_bytes(result.wasted_space)
            )),
        ]));
        lines.push(Line::from(""));

        for (i, group) in result.groups.iter().take(20).enumerate() {
            let selected = i == app.analysis_selected;
            let prefix = if selected { ">" } else { " " };
            lines.push(Line::from(format!(
                " {} Group {} ({} files, {} each)",
                prefix,
                i + 1,
                group.files.len(),
                crate::utils::format_bytes(group.size)
            )));
            for path in group.files.iter().take(3) {
                lines.push(Line::from(format!(
                    "     {}",
                    crate::utils::truncate_str(&path.to_string_lossy(), 60)
                )));
            }
            if group.files.len() > 3 {
                lines.push(Line::from(format!("     ... and {} more", group.files.len() - 3)));
            }
            lines.push(Line::from(""));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No duplicates found"));
    }

    lines
}

/// Generate file type analysis content
fn file_type_analysis_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref analysis) = app.file_type_analysis {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Total: ", styles::accent()),
            Span::raw(format!(
                "{} files, {}",
                analysis.total_files,
                crate::utils::format_bytes(analysis.total_size)
            )),
        ]));
        lines.push(Line::from(""));

        for (category, stats) in analysis.categories_by_size() {
            let pct = analysis.size_percentage(category);
            lines.push(Line::from(format!(
                "   {}: {} files, {} ({:.1}%)",
                category.label(),
                stats.count,
                crate::utils::format_bytes(stats.size),
                pct
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No file type data"));
    }

    lines
}

/// Generate age analysis content
fn age_analysis_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref analysis) = app.age_analysis {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Old Files (>1 year): ", styles::accent()),
            Span::raw(format!(
                "{} files, {}",
                analysis.old_files.len(),
                crate::utils::format_bytes(analysis.old_files_size())
            )),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(" By Age:"));
        for (bucket, stats) in analysis.mod_buckets_chronological() {
            lines.push(Line::from(format!(
                "   {}: {} files, {}",
                bucket.label(),
                stats.count,
                crate::utils::format_bytes(stats.size)
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(" Top Old Files:"));
        for old in analysis.top_old_files(10) {
            lines.push(Line::from(format!(
                "   {} - {} ({})",
                crate::utils::truncate_str(&old.path.to_string_lossy(), 40),
                crate::utils::format_bytes(old.size),
                crate::analyzer::format_age(old.mod_age_days)
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No age data"));
    }

    lines
}

/// Generate large files content
fn large_files_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref files) = app.large_files {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Large Files: ", styles::accent()),
            Span::raw(format!("{} files", files.len())),
        ]));
        lines.push(Line::from(""));

        for (i, file) in files.iter().take(50).enumerate() {
            let selected = i == app.analysis_selected;
            let prefix = if selected { ">" } else { " " };
            lines.push(Line::from(format!(
                " {} {} - {}",
                prefix,
                crate::utils::format_bytes(file.size),
                crate::utils::truncate_str(&file.path.to_string_lossy(), 50)
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No large files data"));
    }

    lines
}

/// Generate cache view content
fn cache_view_content(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref caches) = app.system_caches {
        let total: u64 = caches.iter().filter_map(|c| c.size).sum();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" System Caches: ", styles::accent()),
            Span::raw(format!("{} locations, {}", caches.len(), crate::utils::format_bytes(total))),
        ]));
        lines.push(Line::from(""));

        for (i, cache) in caches.iter().enumerate() {
            let selected = i == app.analysis_selected;
            let prefix = if selected { ">" } else { " " };
            let size_str = cache.size.map(crate::utils::format_bytes).unwrap_or_default();
            lines.push(Line::from(format!(
                " {} {} ({}) - {}",
                prefix,
                cache.name,
                cache.category.label(),
                size_str
            )));
            lines.push(Line::from(format!(
                "     {}",
                cache.path.display()
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(" No cache data"));
    }

    lines
}