use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{ActiveTab, App};

/// Render the header bar showing branch, MR info, and tab navigation.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)));

    if let Some(ref mr) = app.mr {
        // Build the info line
        let status_color = match mr.status {
            crate::provider::MrStatus::Open => Color::Green,
            crate::provider::MrStatus::Merged => Color::Magenta,
            crate::provider::MrStatus::Closed => Color::Red,
        };

        let pipeline_span = if let Some(ref status) = mr.pipeline_status {
            let (symbol, color) = match status.as_str() {
                "success" => ("● ", Color::Green),
                "failed" => ("✗ ", Color::Red),
                "running" => ("◉ ", Color::Yellow),
                "pending" => ("○ ", Color::DarkGray),
                "canceled" => ("⊘ ", Color::DarkGray),
                _ => ("? ", Color::White),
            };
            vec![
                Span::raw(" │ "),
                Span::styled(format!("{}{}", symbol, status), Style::default().fg(color)),
            ]
        } else {
            vec![]
        };

        let mut spans = vec![
            Span::styled(
                format!(" {} ", app.branch),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(130, 170, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("!{}", mr.iid),
                Style::default()
                    .fg(Color::Rgb(200, 200, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(&mr.title, Style::default().fg(Color::White)),
            Span::raw(" │ "),
            Span::styled(
                format!("@{}", mr.author),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" │ "),
            Span::styled(mr.status.to_string(), Style::default().fg(status_color)),
        ];
        spans.extend(pipeline_span);

        // Tab indicators
        let tab_line = build_tab_line(app);

        let content = vec![Line::from(spans), tab_line];

        let header = Paragraph::new(content).block(block);
        frame.render_widget(header, area);
    } else {
        let loading_text = if app.loading {
            "Loading..."
        } else {
            "No MR found"
        };

        let spans = vec![
            Span::styled(
                format!(" {} ", app.branch),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(130, 170, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                loading_text,
                Style::default().fg(Color::DarkGray).italic(),
            ),
        ];

        let tab_line = build_tab_line(app);
        let content = vec![Line::from(spans), tab_line];

        let header = Paragraph::new(content).block(block);
        frame.render_widget(header, area);
    }
}

/// Build the tab indicator line.
fn build_tab_line(app: &App) -> Line<'static> {
    let tabs: Vec<Span> = ActiveTab::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let is_active = *tab == app.active_tab;
            let label = tab.label().to_string();
            let mut spans = vec![];

            if i > 0 {
                spans.push(Span::styled(
                    " │ ",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if is_active {
                spans.push(Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .fg(Color::Rgb(130, 170, 255))
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {} ", label),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            spans
        })
        .collect();

    Line::from(tabs)
}
