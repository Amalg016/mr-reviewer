mod app;
mod diff;
mod event;
mod git;
mod input;
mod provider;
mod ui;

use std::time::Duration;

use anyhow::{Context, Result};

use app::App;
use event::{AppEvent, EventHandler};
use provider::gitlab::GitLabProvider;
use provider::MrProvider;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Detect branch and project from git remote
    let branch = git::get_current_branch()
        .context("Failed to detect git branch. Are you in a git repository?")?;

    let (project, base_url) = git::detect_project_from_remote()
        .context("Failed to detect GitLab project from git remote")?;

    // 2. Create provider
    let provider = GitLabProvider::new(project, base_url);

    // 3. Install panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = ratatui::restore();
        original_hook(panic_info);
    }));

    // 4. Initialize terminal
    let mut terminal = ratatui::init();

    // 5. Create app state
    let mut app = App::new(branch.clone());

    // 6. Spawn async task to fetch MR data
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataUpdate>();

    {
        let branch = branch.clone();
        let provider = provider.clone();
        let tx = data_tx.clone();

        tokio::spawn(async move {
            // Fetch MRs for branch
            match provider.fetch_mr_for_branch(&branch).await {
                Ok(mrs) => {
                    if let Some(mr) = mrs.into_iter().next() {
                        let _ = tx.send(DataUpdate::MrLoaded(mr));
                    } else {
                        let _ = tx.send(DataUpdate::NoMrFound);
                    }
                }
                Err(e) => {
                    let _ = tx.send(DataUpdate::Error(format!("Failed to fetch MR: {}", e)));
                }
            }
        });
    }

    // 7. Event loop
    let mut events = EventHandler::new(Duration::from_millis(50));

    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Check for async data updates (non-blocking)
        while let Ok(update) = data_rx.try_recv() {
            match update {
                DataUpdate::MrLoaded(mr) => {
                    let iid = mr.iid;
                    app.mr = Some(mr);
                    app.set_status("MR loaded successfully", false);

                    // Fetch diff files and comments
                    let provider = provider.clone();
                    let tx = data_tx.clone();
                    tokio::spawn(async move {
                        match provider.fetch_diff_files(iid).await {
                            Ok(files) => {
                                let _ = tx.send(DataUpdate::DiffFilesLoaded(files));
                            }
                            Err(e) => {
                                let _ = tx.send(DataUpdate::Error(format!(
                                    "Failed to fetch diffs: {}",
                                    e
                                )));
                            }
                        }

                        match provider.fetch_comments(iid).await {
                            Ok(comments) => {
                                let _ = tx.send(DataUpdate::CommentsLoaded(comments));
                            }
                            Err(e) => {
                                let _ = tx.send(DataUpdate::Error(format!(
                                    "Failed to fetch comments: {}",
                                    e
                                )));
                            }
                        }
                    });
                }
                DataUpdate::DiffFilesLoaded(files) => {
                    app.diff_files = files;
                    app.loading = false;
                    app.set_status(
                        format!("{} files changed", app.diff_files.len()),
                        false,
                    );
                }
                DataUpdate::CommentsLoaded(comments) => {
                    app.comments = comments;
                }
                DataUpdate::NoMrFound => {
                    app.loading = false;
                    app.set_status("No open MR found for this branch", false);
                }
                DataUpdate::Error(msg) => {
                    app.loading = false;
                    app.set_status(msg, true);
                }
            }
        }

        // Handle terminal events
        match events.next().await? {
            AppEvent::Key(key) => {
                input::handle_key_event(&mut app, key);
            }
            AppEvent::Resize(_, _) => {
                // ratatui handles resize automatically on next draw
            }
            AppEvent::Tick => {
                // Could handle animations, status message timeout, etc.
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 8. Restore terminal
    ratatui::restore();

    Ok(())
}

/// Messages sent from async data-fetching tasks to the main event loop.
#[derive(Debug)]
enum DataUpdate {
    MrLoaded(provider::MrSummary),
    DiffFilesLoaded(Vec<provider::DiffFile>),
    CommentsLoaded(Vec<provider::MrComment>),
    NoMrFound,
    Error(String),
}
