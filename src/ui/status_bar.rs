use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{ActivePane, AppMode, App};

/// Render the bottom status bar showing mode, context hints, and messages.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span> = Vec::new();

    // Mode indicator
    let (mode_label, mode_fg, mode_bg) = match &app.mode {
        AppMode::Normal => (" NORMAL ", Color::Black, Color::Rgb(130, 170, 255)),
        AppMode::VisualLine { .. } => (" VISUAL ", Color::Black, Color::Rgb(255, 200, 80)),
        AppMode::CommentInput => (" COMMENT ", Color::Black, Color::Rgb(80, 220, 200)),
    };

    spans.push(Span::styled(
        mode_label,
        Style::default()
            .fg(mode_fg)
            .bg(mode_bg)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::raw(" "));

    // Pane indicator
    let pane_label = match app.active_pane {
        ActivePane::FileTree => "Files",
        ActivePane::DiffView => "Diff",
    };
    spans.push(Span::styled(
        format!("[{}]", pane_label),
        Style::default().fg(Color::DarkGray),
    ));

    spans.push(Span::raw(" "));

    // Status message or context-sensitive help
    if let Some(ref msg) = app.status_message {
        let color = if msg.is_error {
            Color::Rgb(240, 80, 80)
        } else {
            Color::Rgb(80, 220, 100)
        };
        spans.push(Span::styled(&msg.text, Style::default().fg(color)));
    } else {
        // Context-sensitive key hints
        let hints = match &app.mode {
            AppMode::Normal => match app.active_pane {
                ActivePane::FileTree => {
                    "j/k: navigate │ Enter: open │ m: reviewed │ Tab: switch pane │ H/L: tabs │ q: quit"
                }
                ActivePane::DiffView => {
                    "j/k: navigate │ c: comment │ v: visual │ m: reviewed │ A: approve │ Tab: switch │ q: quit"
                }
            },
            AppMode::VisualLine { .. } => {
                "j/k: extend selection │ a/Enter: comment │ Esc: cancel"
            }
            AppMode::CommentInput => "Ctrl+S: submit │ Esc: cancel │ Enter: newline",
        };
        spans.push(Span::styled(
            hints,
            Style::default().fg(Color::Rgb(100, 100, 130)),
        ));
    }

    // Loading indicator on the right
    if app.loading {
        // Pad to right-align
        let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
        let indicator = " ⟳ Loading ";
        let remaining = (area.width as usize).saturating_sub(used_width + indicator.len());
        if remaining > 0 {
            spans.push(Span::raw(" ".repeat(remaining)));
        }
        spans.push(Span::styled(
            indicator,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let status_line = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Rgb(25, 25, 35)));

    frame.render_widget(status_line, area);
}
