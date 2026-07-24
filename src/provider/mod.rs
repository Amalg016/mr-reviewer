use serde::Deserialize;
use std::fmt;

pub mod gitlab;

#[derive(Debug, Clone, Deserialize)]
pub enum MrStatus {
    Open,
    Merged,
    Closed,
}

impl fmt::Display for MrStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MrStatus::Open => write!(f, "Open"),
            MrStatus::Merged => write!(f, "Merged"),
            MrStatus::Closed => write!(f, "Closed"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl fmt::Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileStatus::Added => write!(f, "A"),
            FileStatus::Modified => write!(f, "M"),
            FileStatus::Deleted => write!(f, "D"),
            FileStatus::Renamed => write!(f, "R"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MrSummary {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub author: String,
    pub status: MrStatus,
    pub web_url: String,
    pub source_branch: String,
    pub target_branch: String,
    pub pipeline_status: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DiffFile {
    pub old_path: String,
    pub new_path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub diff_content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MrComment {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub file_path: Option<String>,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub resolved: bool,
    pub created_at: String,
}

#[allow(dead_code)]
pub trait MrProvider {
    async fn fetch_mr_for_branch(&self, branch: &str) -> anyhow::Result<Vec<MrSummary>>;
    async fn fetch_diff_files(&self, mr_iid: u64) -> anyhow::Result<Vec<DiffFile>>;
    async fn fetch_comments(&self, mr_iid: u64) -> anyhow::Result<Vec<MrComment>>;
    async fn post_comment(
        &self,
        mr_iid: u64,
        body: &str,
        file_path: Option<&str>,
        new_line: Option<usize>,
    ) -> anyhow::Result<()>;
    async fn approve_mr(&self, mr_iid: u64) -> anyhow::Result<()>;
}
