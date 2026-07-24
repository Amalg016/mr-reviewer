# mr-reviewer

A fast, keyboard-driven Terminal User Interface (TUI) tool written in Rust with [`ratatui`](https://crates.io/crates/ratatui) for reviewing Merge Requests / Pull Requests directly from the command line without browser lag.

![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

---

## Features

- **Vim-like Navigation**: Navigate files and diffs effortlessly with `j`/`k`, `g`/`G`, `Ctrl+d`/`Ctrl+u`, visual selection, and line targeting.
- **Multi-Pane Layout**:
  - **Header**: Displays active branch, MR title, author, MR status, and CI/CD pipeline indicators.
  - **File Tree Sidebar**: Shows changed files with status badges (`M` modified, `A` added, `D` deleted, `R` renamed), diff statistics (`+lines`, `-lines`), and reviewed checkmarks (`✓`).
  - **Unified Diff Viewer**: Dual old/new line numbers with color-coded diff additions, deletions, cursor line highlighting, and visual mode selection.
  - **Discussions & Comment Overlay**: View existing review threads or compose new comments in a floating popup overlay.
- **Git Remote Auto-Detection**: Detects current branch and GitLab project automatically from your git remote.
- **Responsive & Async**: Built with `tokio` so network calls never freeze the terminal UI.
- **Extensible Provider Architecture**: Abstract `MrProvider` trait for easy addition of other Git platforms (GitHub, Gitea, etc.).

---

## Keybindings

### Navigation (Normal Mode)

| Keybinding | Action |
|------------|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `Ctrl+d` | Scroll half-page down |
| `Ctrl+u` | Scroll half-page up |
| `Tab` | Toggle focus between **Files** sidebar and **Diff** pane |
| `H` / `←` | Switch to previous tab |
| `L` / `→` | Switch to next tab |
| `Enter` | Open selected file in file tree |

### Actions & Review Workflow

| Keybinding | Action |
|------------|--------|
| `c` | Open inline comment overlay on current line |
| `v` | Enter **Visual Line** selection mode |
| `m` | Toggle current file as **Reviewed / Viewed** (`✓`) |
| `Shift+A` | Request MR approval |
| `q` | Quit application |
| `Esc` | Clear status / cancel action |

### Visual Line Mode (`v`)

| Keybinding | Action |
|------------|--------|
| `j` / `k` | Extend line selection down / up |
| `a` / `Enter` | Open comment overlay for selected line range |
| `Esc` | Exit Visual Line mode |

### Comment Input Overlay

| Keybinding | Action |
|------------|--------|
| `Ctrl+S` | Submit comment |
| `Enter` | Add new line |
| `Backspace` | Delete character |
| `Esc` | Cancel comment |

---

## Installation & Setup

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75+)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/mr-reviewer.git
cd mr-reviewer

# Build and run
cargo run
```

### Installation

To install `mr-reviewer` binary globally on your path:

```bash
cargo install --path .
```

---

## Configuration

For private GitLab repositories, export your personal access token:

```bash
export GITLAB_TOKEN="your-gitlab-personal-access-token"
```

Then run `mr-reviewer` inside any local git repository:

```bash
cd /path/to/your/gitlab-project
mr-reviewer
```

---

## Architecture Overview

```
src/
├── main.rs            # Application entry point & async event loop
├── app.rs             # App state engine & mode manager (Normal, Visual, Comment)
├── event.rs           # Crossterm terminal event handler
├── input.rs           # Keybinding dispatcher
├── git.rs             # Git CLI helpers & remote URL parser
├── diff.rs            # Unified diff parser & hunk line calculator
├── provider/
│   ├── mod.rs         # MrProvider trait & domain models
│   └── gitlab.rs      # GitLab REST API client
└── ui/
    ├── mod.rs         # Layout composition & overview tab
    ├── header.rs      # Header bar & pipeline status
    ├── file_tree.rs   # Changed files sidebar
    ├── diff_view.rs   # Unified diff renderer & cursor highlights
    ├── comment.rs     # Comment popup overlay & discussions tab
    └── status_bar.rs  # Bottom mode bar & keyboard hints
```

---

## License

MIT License.
