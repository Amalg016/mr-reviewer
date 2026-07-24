use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActivePane, AppMode, App};

/// Top-level key event dispatcher. Routes to the appropriate mode handler.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::VisualLine { .. } => handle_visual_mode(app, key),
        AppMode::CommentInput => handle_comment_mode(app, key),
    }
}

/// Handle key events in Normal mode.
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // ── Quit ────────────────────────────────────────
        (KeyCode::Char('q'), KeyModifiers::NONE) => {
            app.should_quit = true;
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }

        // ── Vertical navigation ─────────────────────────
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            app.move_down();
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
            app.move_up();
        }

        // ── Jump to top / bottom ────────────────────────
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            app.goto_top();
        }
        (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::Char('G'), KeyModifiers::NONE) => {
            app.goto_bottom();
        }

        // ── Half-page scroll ────────────────────────────
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            app.half_page_down();
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.half_page_up();
        }

        // ── Pane switching ──────────────────────────────
        (KeyCode::Tab, _) => {
            app.toggle_pane();
        }

        // ── Tab switching ───────────────────────────────
        (KeyCode::Char('L'), KeyModifiers::SHIFT) | (KeyCode::Char('L'), KeyModifiers::NONE) => {
            // Uppercase L → next tab (only if it's truly uppercase)
            if key.code == KeyCode::Char('L') {
                app.next_tab();
            }
        }
        (KeyCode::Char('H'), KeyModifiers::SHIFT) | (KeyCode::Char('H'), KeyModifiers::NONE) => {
            if key.code == KeyCode::Char('H') {
                app.prev_tab();
            }
        }
        (KeyCode::Right, _) => {
            app.next_tab();
        }
        (KeyCode::Left, _) => {
            app.prev_tab();
        }

        // ── File selection (Enter) ──────────────────────
        (KeyCode::Enter, _) => {
            if app.active_pane == ActivePane::FileTree {
                app.select_current_file();
            }
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) => {
            // lowercase l in file tree → open file (like vim right-motion)
            if app.active_pane == ActivePane::FileTree {
                app.select_current_file();
            }
        }

        // ── Visual mode ─────────────────────────────────
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            app.enter_visual_mode();
        }

        // ── Comment on current line ─────────────────────
        (KeyCode::Char('c'), KeyModifiers::NONE) => {
            app.start_comment_on_cursor();
        }

        // ── Toggle reviewed ─────────────────────────────
        (KeyCode::Char('m'), KeyModifiers::NONE) => {
            app.toggle_reviewed();
        }

        // ── Approve MR ─────────────────────────────────
        (KeyCode::Char('A'), KeyModifiers::SHIFT) | (KeyCode::Char('A'), KeyModifiers::NONE) => {
            if key.code == KeyCode::Char('A') {
                app.set_status("MR approval requested (not yet implemented)", false);
            }
        }

        // ── Escape clears status ────────────────────────
        (KeyCode::Esc, _) => {
            app.clear_status();
        }

        _ => {}
    }
}

/// Handle key events in Visual Line selection mode.
fn handle_visual_mode(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // ── Extend selection ────────────────────────────
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            if app.diff_cursor + 1 < app.diff_lines.len() {
                app.diff_cursor += 1;
            }
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
            app.diff_cursor = app.diff_cursor.saturating_sub(1);
        }

        // ── Half-page scroll in visual mode ─────────────
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            let half = app.viewport_height / 2;
            let max = app.diff_lines.len().saturating_sub(1);
            app.diff_cursor = (app.diff_cursor + half).min(max);
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            let half = app.viewport_height / 2;
            app.diff_cursor = app.diff_cursor.saturating_sub(half);
        }

        // ── Open comment for selection ──────────────────
        (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Enter, _) => {
            app.start_comment_on_selection();
        }

        // ── Cancel visual mode ──────────────────────────
        (KeyCode::Esc, _) | (KeyCode::Char('v'), KeyModifiers::NONE) => {
            app.exit_visual_mode();
        }

        _ => {}
    }
}

/// Handle key events in Comment Input mode.
fn handle_comment_mode(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // ── Submit comment ──────────────────────────────
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            if app.comment_buffer.trim().is_empty() {
                app.set_status("Comment is empty", true);
            } else {
                // TODO: Submit via provider API
                app.set_status("Comment submitted (not yet implemented)", false);
                app.cancel_comment();
            }
        }

        // ── Cancel ──────────────────────────────────────
        (KeyCode::Esc, _) => {
            app.cancel_comment();
        }

        // ── New line ────────────────────────────────────
        (KeyCode::Enter, _) => {
            app.comment_buffer.push('\n');
        }

        // ── Backspace ───────────────────────────────────
        (KeyCode::Backspace, _) => {
            app.comment_buffer.pop();
        }

        // ── Type character ──────────────────────────────
        (KeyCode::Char(c), _) => {
            app.comment_buffer.push(c);
        }

        _ => {}
    }
}
