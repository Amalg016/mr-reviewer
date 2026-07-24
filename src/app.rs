use std::collections::HashSet;

use ratatui::widgets::ListState;

use crate::diff::DiffLine;
use crate::provider::{DiffFile, MrComment, MrSummary};

/// The current input mode of the application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Standard navigation mode (Vim normal mode).
    Normal,
    /// Visual line selection mode. `anchor` is the line index where `v` was pressed.
    VisualLine { anchor: usize },
    /// Inline comment composition mode.
    CommentInput,
}

/// Which pane has focus in the Changes tab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePane {
    FileTree,
    DiffView,
}

/// Top-level tab selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Overview,
    Changes,
    Discussions,
}

impl ActiveTab {
    pub const ALL: [ActiveTab; 3] = [
        ActiveTab::Overview,
        ActiveTab::Changes,
        ActiveTab::Discussions,
    ];

    pub fn index(self) -> usize {
        match self {
            ActiveTab::Overview => 0,
            ActiveTab::Changes => 1,
            ActiveTab::Discussions => 2,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 3 {
            0 => ActiveTab::Overview,
            1 => ActiveTab::Changes,
            2 => ActiveTab::Discussions,
            _ => unreachable!(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Overview => "Overview",
            ActiveTab::Changes => "Changes",
            ActiveTab::Discussions => "Discussions",
        }
    }
}

/// Central application state.
pub struct App {
    // -- Mode & navigation --
    pub mode: AppMode,
    pub active_pane: ActivePane,
    pub active_tab: ActiveTab,
    pub should_quit: bool,

    // -- Data (loaded asynchronously) --
    pub branch: String,
    pub mr: Option<MrSummary>,
    pub diff_files: Vec<DiffFile>,
    pub comments: Vec<MrComment>,
    pub loading: bool,
    pub status_message: Option<StatusMessage>,

    // -- File tree state --
    pub file_list_state: ListState,

    // -- Diff view state --
    pub diff_lines: Vec<DiffLine>,
    pub diff_scroll: usize,
    pub diff_cursor: usize,
    pub viewport_height: usize,

    // -- Visual selection --
    pub visual_anchor: Option<usize>,

    // -- Comment input --
    pub comment_buffer: String,
    pub comment_target_lines: (usize, usize),

    // -- Reviewed file tracking --
    pub reviewed_files: HashSet<usize>,

    // -- Currently selected file index --
    pub selected_file_index: Option<usize>,
}

/// A timed status message shown in the status bar.
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

impl App {
    pub fn new(branch: String) -> Self {
        let mut file_list_state = ListState::default();
        file_list_state.select(Some(0));

        Self {
            mode: AppMode::Normal,
            active_pane: ActivePane::FileTree,
            active_tab: ActiveTab::Changes,
            should_quit: false,

            branch,
            mr: None,
            diff_files: Vec::new(),
            comments: Vec::new(),
            loading: true,
            status_message: None,

            file_list_state,

            diff_lines: Vec::new(),
            diff_scroll: 0,
            diff_cursor: 0,
            viewport_height: 20,

            visual_anchor: None,

            comment_buffer: String::new(),
            comment_target_lines: (0, 0),

            reviewed_files: HashSet::new(),
            selected_file_index: None,
        }
    }

    // ── Navigation ──────────────────────────────────────────

    /// Move cursor down by one line in the current context.
    pub fn move_down(&mut self) {
        match self.active_pane {
            ActivePane::FileTree => {
                if !self.diff_files.is_empty() {
                    self.file_list_state.select_next();
                }
            }
            ActivePane::DiffView => {
                if self.diff_cursor + 1 < self.diff_lines.len() {
                    self.diff_cursor += 1;
                    self.ensure_cursor_visible();
                }
            }
        }
    }

    /// Move cursor up by one line in the current context.
    pub fn move_up(&mut self) {
        match self.active_pane {
            ActivePane::FileTree => {
                self.file_list_state.select_previous();
            }
            ActivePane::DiffView => {
                self.diff_cursor = self.diff_cursor.saturating_sub(1);
                self.ensure_cursor_visible();
            }
        }
    }

    /// Jump to the top.
    pub fn goto_top(&mut self) {
        match self.active_pane {
            ActivePane::FileTree => {
                self.file_list_state.select(Some(0));
            }
            ActivePane::DiffView => {
                self.diff_cursor = 0;
                self.diff_scroll = 0;
            }
        }
    }

    /// Jump to the bottom.
    pub fn goto_bottom(&mut self) {
        match self.active_pane {
            ActivePane::FileTree => {
                if !self.diff_files.is_empty() {
                    self.file_list_state
                        .select(Some(self.diff_files.len() - 1));
                }
            }
            ActivePane::DiffView => {
                if !self.diff_lines.is_empty() {
                    self.diff_cursor = self.diff_lines.len() - 1;
                    self.ensure_cursor_visible();
                }
            }
        }
    }

    /// Scroll half a page down.
    pub fn half_page_down(&mut self) {
        let half = self.viewport_height / 2;
        match self.active_pane {
            ActivePane::FileTree => {
                for _ in 0..half {
                    self.file_list_state.select_next();
                }
            }
            ActivePane::DiffView => {
                let max = self.diff_lines.len().saturating_sub(1);
                self.diff_cursor = (self.diff_cursor + half).min(max);
                self.ensure_cursor_visible();
            }
        }
    }

    /// Scroll half a page up.
    pub fn half_page_up(&mut self) {
        let half = self.viewport_height / 2;
        match self.active_pane {
            ActivePane::FileTree => {
                for _ in 0..half {
                    self.file_list_state.select_previous();
                }
            }
            ActivePane::DiffView => {
                self.diff_cursor = self.diff_cursor.saturating_sub(half);
                self.ensure_cursor_visible();
            }
        }
    }

    /// Ensure the cursor is within the visible viewport, adjusting scroll if needed.
    fn ensure_cursor_visible(&mut self) {
        if self.viewport_height == 0 {
            return;
        }
        // Cursor above viewport
        if self.diff_cursor < self.diff_scroll {
            self.diff_scroll = self.diff_cursor;
        }
        // Cursor below viewport
        if self.diff_cursor >= self.diff_scroll + self.viewport_height {
            self.diff_scroll = self.diff_cursor - self.viewport_height + 1;
        }
    }

    // ── Pane / Tab switching ────────────────────────────────

    /// Toggle focus between FileTree and DiffView panes.
    pub fn toggle_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::FileTree => ActivePane::DiffView,
            ActivePane::DiffView => ActivePane::FileTree,
        };
    }

    /// Switch to the next tab.
    pub fn next_tab(&mut self) {
        let next = (self.active_tab.index() + 1) % ActiveTab::ALL.len();
        self.active_tab = ActiveTab::from_index(next);
    }

    /// Switch to the previous tab.
    pub fn prev_tab(&mut self) {
        let prev = (self.active_tab.index() + ActiveTab::ALL.len() - 1) % ActiveTab::ALL.len();
        self.active_tab = ActiveTab::from_index(prev);
    }

    // ── File selection ──────────────────────────────────────

    /// Select the currently highlighted file and load its diff.
    pub fn select_current_file(&mut self) {
        let idx = self.file_list_state.selected().unwrap_or(0);
        if idx < self.diff_files.len() {
            self.selected_file_index = Some(idx);
            let raw_diff = &self.diff_files[idx].diff_content;
            self.diff_lines = crate::diff::parse_unified_diff(raw_diff);
            self.diff_cursor = 0;
            self.diff_scroll = 0;
            self.active_pane = ActivePane::DiffView;
        }
    }

    /// Toggle the "reviewed" flag on the currently selected file.
    pub fn toggle_reviewed(&mut self) {
        if let Some(idx) = match self.active_pane {
            ActivePane::FileTree => self.file_list_state.selected(),
            ActivePane::DiffView => self.selected_file_index,
        } {
            if self.reviewed_files.contains(&idx) {
                self.reviewed_files.remove(&idx);
            } else {
                self.reviewed_files.insert(idx);
            }
        }
    }

    // ── Visual mode ─────────────────────────────────────────

    /// Enter visual line selection mode at the current cursor.
    pub fn enter_visual_mode(&mut self) {
        if self.active_pane == ActivePane::DiffView && !self.diff_lines.is_empty() {
            self.visual_anchor = Some(self.diff_cursor);
            self.mode = AppMode::VisualLine {
                anchor: self.diff_cursor,
            };
        }
    }

    /// Exit visual mode, returning to Normal.
    pub fn exit_visual_mode(&mut self) {
        self.visual_anchor = None;
        self.mode = AppMode::Normal;
    }

    /// Get the selected line range in visual mode (inclusive, sorted).
    pub fn visual_selection_range(&self) -> Option<(usize, usize)> {
        self.visual_anchor.map(|anchor| {
            let start = anchor.min(self.diff_cursor);
            let end = anchor.max(self.diff_cursor);
            (start, end)
        })
    }

    // ── Comment input ───────────────────────────────────────

    /// Enter comment input mode for the given line range.
    pub fn start_comment(&mut self, start: usize, end: usize) {
        self.comment_target_lines = (start, end);
        self.comment_buffer.clear();
        self.mode = AppMode::CommentInput;
    }

    /// Start a comment on the current cursor line (single line).
    pub fn start_comment_on_cursor(&mut self) {
        if self.active_pane == ActivePane::DiffView && !self.diff_lines.is_empty() {
            self.start_comment(self.diff_cursor, self.diff_cursor);
        }
    }

    /// Start a comment on the visual selection range.
    pub fn start_comment_on_selection(&mut self) {
        if let Some((start, end)) = self.visual_selection_range() {
            self.start_comment(start, end);
        }
    }

    /// Cancel comment input and return to Normal mode.
    pub fn cancel_comment(&mut self) {
        self.comment_buffer.clear();
        self.visual_anchor = None;
        self.mode = AppMode::Normal;
    }

    /// Set a status message.
    pub fn set_status(&mut self, text: impl Into<String>, is_error: bool) {
        self.status_message = Some(StatusMessage {
            text: text.into(),
            is_error,
        });
    }

    /// Clear the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
