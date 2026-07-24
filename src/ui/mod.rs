pub mod comment;
pub mod diff_view;
pub mod file_tree;
pub mod header;
pub mod status_bar;

use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::{ActiveTab, App};

/// Base background color used throughout the UI to prevent ghosting.
const BG: Color = Color::Rgb(18, 18, 24);

/// Top-level render function. Composes the full layout and dispatches to sub-components.
pub fn render(frame: &mut Frame, app: &mut App) {
    // Paint the entire frame with a solid background to prevent any ghost cells
    let bg_block = Block::default().style(Style::default().bg(BG));
    frame.render_widget(bg_block, frame.area());

    let [header_area, main_area, status_area] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // Render header
    header::render(frame, header_area, app);

    // Render main content based on active tab
    match app.active_tab {
        ActiveTab::Overview => {
            render_overview(frame, main_area, app);
        }
        ActiveTab::Changes => {
            let [sidebar_area, diff_area] = Layout::horizontal([
                Constraint::Percentage(25),
                Constraint::Fill(1),
            ])
            .areas(main_area);

            file_tree::render(frame, sidebar_area, app);

            // Update viewport height for scroll calculations (subtract 2 for borders)
            app.viewport_height = diff_area.height.saturating_sub(2) as usize;

            diff_view::render(frame, diff_area, app);

            // Render comment overlay on top if in CommentInput mode
            if app.mode == crate::app::AppMode::CommentInput {
                comment::render_input_overlay(frame, diff_area, app);
            }
        }
        ActiveTab::Discussions => {
            comment::render_discussions(frame, main_area, app);
        }
    }

    // Render status bar
    status_bar::render(frame, status_area, app);
}

/// Render the Overview tab: MR description, metadata, and general info.
fn render_overview(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::style::Stylize;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{BorderType, Borders, Paragraph, Wrap};

    let mr_info = if let Some(ref mr) = app.mr {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&mr.title, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&mr.author, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    mr.status.to_string(),
                    Style::default().fg(match mr.status {
                        crate::provider::MrStatus::Open => Color::Green,
                        crate::provider::MrStatus::Merged => Color::Magenta,
                        crate::provider::MrStatus::Closed => Color::Red,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&mr.source_branch, Style::default().fg(Color::Yellow)),
                Span::raw(" → "),
                Span::styled(&mr.target_branch, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&mr.web_url, Style::default().fg(Color::Blue)),
            ]),
        ];

        if let Some(ref pipeline) = mr.pipeline_status {
            lines.push(Line::from(vec![
                Span::styled("Pipeline: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    pipeline,
                    Style::default().fg(match pipeline.as_str() {
                        "success" => Color::Green,
                        "failed" => Color::Red,
                        "running" => Color::Yellow,
                        "pending" => Color::DarkGray,
                        _ => Color::White,
                    }),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Files Changed: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.diff_files.len().to_string(),
                Style::default().fg(Color::White),
            ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(
            Span::styled("── Description ──", Style::default().fg(Color::DarkGray)),
        ));
        lines.push(Line::from(""));

        for desc_line in mr.description.lines() {
            lines.push(Line::from(desc_line.to_string()));
        }

        lines
    } else if app.loading {
        vec![Line::from(
            Span::styled("Loading MR data...", Style::default().fg(Color::Yellow)).italic(),
        )]
    } else {
        vec![Line::from(
            Span::styled(
                "No merge request found for this branch.",
                Style::default().fg(Color::DarkGray),
            )
            .italic(),
        )]
    };

    let overview = Paragraph::new(mr_info)
        .block(
            Block::default()
                .title(" Overview ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().bg(BG))
        .wrap(Wrap { trim: false });

    frame.render_widget(overview, area);
}
