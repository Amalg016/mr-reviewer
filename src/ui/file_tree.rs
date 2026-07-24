use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, HighlightSpacing, List, ListItem};
use ratatui::Frame;

use crate::app::{ActivePane, App};
use super::BG;

/// Render the file tree sidebar listing changed files with status indicators.
pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_focused = app.active_pane == ActivePane::FileTree;

    let border_color = if is_focused {
        Color::Rgb(130, 170, 255)
    } else {
        Color::Rgb(60, 60, 80)
    };

    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    if app.diff_files.is_empty() {
        let empty_msg = if app.loading {
            "Loading..."
        } else {
            "No changed files"
        };
        let paragraph = ratatui::widgets::Paragraph::new(
            Span::styled(empty_msg, Style::default().fg(Color::DarkGray).italic()),
        )
        .block(block)
        .style(Style::default().bg(BG));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .diff_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let (status_char, status_color) = match file.status {
                crate::provider::FileStatus::Modified => ("M", Color::Yellow),
                crate::provider::FileStatus::Added => ("A", Color::Green),
                crate::provider::FileStatus::Deleted => ("D", Color::Red),
                crate::provider::FileStatus::Renamed => ("R", Color::Blue),
            };

            let reviewed = if app.reviewed_files.contains(&i) {
                " ✓"
            } else {
                ""
            };

            // Extract just the filename for compact display
            let display_name = file
                .new_path
                .rsplit('/')
                .next()
                .unwrap_or(&file.new_path);

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_char),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(display_name.to_string(), Style::default().fg(Color::White)),
                Span::styled(
                    format!(" +{}", file.additions),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" -{}", file.deletions),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    reviewed.to_string(),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().bg(BG))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 70))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.file_list_state);
}
