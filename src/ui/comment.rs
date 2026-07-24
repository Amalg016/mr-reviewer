use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use super::BG;

/// Render the comment input overlay as a floating popup over the diff view.
pub fn render_input_overlay(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate popup area (centered, 60% width, 40% height)
    let popup_area = centered_rect(60, 40, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let (start, end) = app.comment_target_lines;
    let title = if start == end {
        format!(" Comment on line {} ", start + 1)
    } else {
        format!(" Comment on lines {}-{} ", start + 1, end + 1)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(130, 170, 255)));

    // Build the comment buffer display with cursor
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "Type your comment. ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            "Ctrl+S",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to submit, ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to cancel.", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    // Show the comment buffer content
    if app.comment_buffer.is_empty() {
        lines.push(Line::from(Span::styled(
            "│",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::SLOW_BLINK),
        )));
    } else {
        for buf_line in app.comment_buffer.lines() {
            lines.push(Line::from(Span::styled(
                buf_line.to_string(),
                Style::default().fg(Color::White),
            )));
        }
        // Add cursor at end
        if app.comment_buffer.ends_with('\n') || app.comment_buffer.is_empty() {
            lines.push(Line::from(Span::styled(
                "│",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::SLOW_BLINK),
            )));
        } else {
            // Append cursor to last line
            if let Some(last) = lines.last_mut() {
                let spans = last.spans.clone();
                let mut new_spans = spans;
                new_spans.push(Span::styled(
                    "│",
                    Style::default()
                        .fg(Color::Rgb(130, 170, 255))
                        .add_modifier(Modifier::SLOW_BLINK),
                ));
                *last = Line::from(new_spans);
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(BG))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// Render the Discussions tab, showing all comment threads.
pub fn render_discussions(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Discussions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 80)));

    if app.comments.is_empty() {
        let msg = if app.loading {
            "Loading discussions..."
        } else {
            "No discussions on this MR."
        };
        let paragraph = Paragraph::new(Span::styled(
            msg,
            Style::default().fg(Color::DarkGray).italic(),
        ))
        .block(block)
        .style(Style::default().bg(BG));
        frame.render_widget(paragraph, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    for comment in &app.comments {
        // Comment header
        let resolved_indicator = if comment.resolved {
            Span::styled(" ✓ resolved", Style::default().fg(Color::Green))
        } else {
            Span::styled(" ○ open", Style::default().fg(Color::Yellow))
        };

        let location = if let Some(ref path) = comment.file_path {
            let line_info = comment
                .new_line
                .map(|l| format!(":{}", l))
                .unwrap_or_default();
            format!(" @ {}{}", path, line_info)
        } else {
            String::new()
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("@{}", comment.author),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                location,
                Style::default().fg(Color::Rgb(130, 130, 180)),
            ),
            resolved_indicator,
        ]));

        // Comment body
        for body_line in comment.body.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", body_line),
                Style::default().fg(Color::White),
            )));
        }

        // Separator
        lines.push(Line::from(Span::styled(
            "  ─────────────────────────────",
            Style::default().fg(Color::Rgb(50, 50, 70)),
        )));
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(BG))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Helper: compute a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, vertical_center, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);

    let [_, popup_area, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(vertical_center);

    popup_area
}
