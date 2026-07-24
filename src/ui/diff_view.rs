use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use crate::app::{ActivePane, AppMode, App};
use crate::diff::DiffLineKind;

use super::BG;

/// Render the unified diff view with line numbers, syntax coloring, cursor, and visual selection.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.active_pane == ActivePane::DiffView;

    let border_color = if is_focused {
        Color::Rgb(130, 170, 255)
    } else {
        Color::Rgb(60, 60, 80)
    };

    let title = if let Some(idx) = app.selected_file_index {
        if idx < app.diff_files.len() {
            format!(" {} ", app.diff_files[idx].new_path)
        } else {
            " Diff ".to_string()
        }
    } else {
        " Diff ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    if app.diff_lines.is_empty() {
        let empty_msg = if app.selected_file_index.is_some() {
            "Empty diff"
        } else {
            "Select a file to view diff (press Enter)"
        };
        let paragraph = Paragraph::new(
            Span::styled(empty_msg, Style::default().fg(Color::DarkGray).italic()),
        )
        .block(block)
        .style(Style::default().bg(BG));
        frame.render_widget(paragraph, area);
        return;
    }

    // Compute visual selection range
    let visual_range = match app.mode {
        AppMode::VisualLine { .. } => app.visual_selection_range(),
        _ => None,
    };

    // Build styled lines for the visible viewport
    let lines: Vec<Line> = app
        .diff_lines
        .iter()
        .enumerate()
        .map(|(i, dl)| {
            let is_cursor_line = i == app.diff_cursor && is_focused;
            let is_selected = visual_range
                .map(|(start, end)| i >= start && i <= end)
                .unwrap_or(false);

            // Line number columns
            let old_no = dl
                .old_lineno
                .map_or("    ".to_string(), |n| format!("{:4}", n));
            let new_no = dl
                .new_lineno
                .map_or("    ".to_string(), |n| format!("{:4}", n));

            // Colors based on diff line kind
            let (prefix, fg_color) = match dl.kind {
                DiffLineKind::FileHeader => ("", Color::Rgb(180, 180, 255)),
                DiffLineKind::HunkHeader => ("", Color::Cyan),
                DiffLineKind::Addition => ("+", Color::Rgb(80, 220, 100)),
                DiffLineKind::Deletion => ("-", Color::Rgb(240, 80, 80)),
                DiffLineKind::Context => (" ", Color::Rgb(160, 160, 160)),
            };

            // Background for cursor / visual selection — always explicit, never Reset
            let bg_color = if is_cursor_line && is_selected {
                Color::Rgb(60, 60, 120)
            } else if is_cursor_line {
                Color::Rgb(45, 45, 65)
            } else if is_selected {
                Color::Rgb(50, 50, 90)
            } else {
                match dl.kind {
                    DiffLineKind::Addition => Color::Rgb(15, 35, 15),
                    DiffLineKind::Deletion => Color::Rgb(40, 15, 15),
                    _ => BG,
                }
            };

            let line_style = Style::default().fg(fg_color).bg(bg_color);
            let gutter_style = Style::default().fg(Color::Rgb(90, 90, 110)).bg(bg_color);
            let prefix_style = Style::default()
                .fg(fg_color)
                .bg(bg_color)
                .add_modifier(Modifier::BOLD);

            match dl.kind {
                DiffLineKind::FileHeader | DiffLineKind::HunkHeader => {
                    // Full-width display for headers
                    Line::from(vec![
                        Span::styled("         ", gutter_style),
                        Span::styled(&dl.content, line_style.add_modifier(Modifier::BOLD)),
                    ]).style(Style::default().bg(bg_color))
                }
                _ => Line::from(vec![
                    Span::styled(old_no, gutter_style),
                    Span::styled("|", Style::default().fg(Color::Rgb(50, 50, 70)).bg(bg_color)),
                    Span::styled(new_no, gutter_style),
                    Span::styled("|", Style::default().fg(Color::Rgb(50, 50, 70)).bg(bg_color)),
                    Span::styled(format!("{} ", prefix), prefix_style),
                    Span::styled(&dl.content, line_style),
                ]).style(Style::default().bg(bg_color)),
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(BG))
        .scroll((app.diff_scroll as u16, 0));

    frame.render_widget(paragraph, area);

    // Render scrollbar
    if app.diff_lines.len() > app.viewport_height {
        let mut scrollbar_state = ScrollbarState::new(app.diff_lines.len())
            .position(app.diff_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"))
            .track_symbol(Some("|"))
            .thumb_symbol("#");

        // Render scrollbar inside the block border area
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
