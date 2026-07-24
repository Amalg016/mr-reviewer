use crate::provider::{DiffFile, FileStatus, MrComment, MrProvider, MrStatus, MrSummary};
use anyhow::{Context, Result};
use reqwest::{header, Client};
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug)]
pub struct GitLabProvider {
    client: Client,
    project: String,
    base_url: String,
}

impl GitLabProvider {
    pub fn new(project: String, base_url: String) -> Self {
        let mut headers = header::HeaderMap::new();
        if let Ok(token) = env::var("GITLAB_TOKEN") {
            if let Ok(value) = header::HeaderValue::from_str(&token) {
                headers.insert("PRIVATE-TOKEN", value);
            }
        }
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new());

        let project_encoded = urlencoding::encode(&project).into_owned();

        Self {
            client,
            project: project_encoded,
            base_url,
        }
    }
}

// API Structs
#[derive(Deserialize, Debug)]
struct GlAuthor {
    username: String,
}

#[derive(Deserialize, Debug)]
struct GlPipeline {
    status: String,
}

#[derive(Deserialize, Debug)]
struct GlMergeRequest {
    id: u64,
    iid: u64,
    title: String,
    state: String,
    web_url: String,
    source_branch: String,
    target_branch: String,
    #[serde(default)]
    description: String,
    author: GlAuthor,
    head_pipeline: Option<GlPipeline>,
}

#[derive(Deserialize, Debug)]
struct GlChange {
    old_path: String,
    new_path: String,
    new_file: bool,
    renamed_file: bool,
    deleted_file: bool,
    diff: String,
}

#[derive(Deserialize, Debug)]
struct GlMergeRequestChanges {
    changes: Vec<GlChange>,
}

#[derive(Deserialize, Debug)]
struct GlPosition {
    new_path: Option<String>,
    new_line: Option<usize>,
    old_line: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct GlNote {
    id: u64,
    author: GlAuthor,
    body: String,
    #[serde(default)]
    resolved: bool,
    created_at: String,
    position: Option<GlPosition>,
    #[serde(default)]
    system: bool,
}

#[derive(Deserialize, Debug)]
struct GlDiscussion {
    notes: Vec<GlNote>,
}

impl MrProvider for GitLabProvider {
    async fn fetch_mr_for_branch(&self, branch: &str) -> Result<Vec<MrSummary>> {
        let url = format!("{}/api/v4/projects/{}/merge_requests", self.base_url, self.project);
        let resp = self.client.get(&url)
            .query(&[("source_branch", branch), ("state", "opened")])
            .send()
            .await
            .context("Failed to send request for MRs")?
            .error_for_status()
            .context("Error response for MRs")?;

        let gl_mrs: Vec<GlMergeRequest> = resp.json().await.context("Failed to parse MRs")?;

        let summaries = gl_mrs.into_iter().map(|mr| {
            let status = match mr.state.as_str() {
                "opened" => MrStatus::Open,
                "merged" => MrStatus::Merged,
                "closed" => MrStatus::Closed,
                _ => MrStatus::Open,
            };

            MrSummary {
                id: mr.id,
                iid: mr.iid,
                title: mr.title,
                author: mr.author.username,
                status,
                web_url: mr.web_url,
                source_branch: mr.source_branch,
                target_branch: mr.target_branch,
                pipeline_status: mr.head_pipeline.map(|p| p.status),
                description: mr.description,
            }
        }).collect();

        Ok(summaries)
    }

    async fn fetch_diff_files(&self, mr_iid: u64) -> Result<Vec<DiffFile>> {
        let url = format!("{}/api/v4/projects/{}/merge_requests/{}/changes", self.base_url, self.project, mr_iid);
        let resp = self.client.get(&url)
            .send()
            .await
            .context("Failed to send request for diffs")?
            .error_for_status()
            .context("Error response for diffs")?;

        let mr_changes: GlMergeRequestChanges = resp.json().await.context("Failed to parse diffs")?;

        let diff_files = mr_changes.changes.into_iter().map(|change| {
            let status = if change.new_file {
                FileStatus::Added
            } else if change.deleted_file {
                FileStatus::Deleted
            } else if change.renamed_file {
                FileStatus::Renamed
            } else {
                FileStatus::Modified
            };

            let mut additions = 0;
            let mut deletions = 0;
            for line in change.diff.lines() {
                if line.starts_with('+') && !line.starts_with("+++") {
                    additions += 1;
                } else if line.starts_with('-') && !line.starts_with("---") {
                    deletions += 1;
                }
            }

            DiffFile {
                old_path: change.old_path,
                new_path: change.new_path,
                status,
                additions,
                deletions,
                diff_content: change.diff,
            }
        }).collect();

        Ok(diff_files)
    }

    async fn fetch_comments(&self, mr_iid: u64) -> Result<Vec<MrComment>> {
        let url = format!("{}/api/v4/projects/{}/merge_requests/{}/discussions", self.base_url, self.project, mr_iid);
        let resp = self.client.get(&url)
            .send()
            .await
            .context("Failed to send request for discussions")?
            .error_for_status()
            .context("Error response for discussions")?;

        let discussions: Vec<GlDiscussion> = resp.json().await.context("Failed to parse discussions")?;

        let mut comments = Vec::new();
        for discussion in discussions {
            for note in discussion.notes {
                if note.system { continue; }

                let mut file_path = None;
                let mut old_line = None;
                let mut new_line = None;

                if let pos @ Some(_) = note.position {
                    let pos = pos.unwrap();
                    file_path = pos.new_path;
                    old_line = pos.old_line;
                    new_line = pos.new_line;
                }

                comments.push(MrComment {
                    id: note.id,
                    author: note.author.username,
                    body: note.body,
                    file_path,
                    old_line,
                    new_line,
                    resolved: note.resolved,
                    created_at: note.created_at,
                });
            }
        }

        Ok(comments)
    }

    async fn post_comment(
        &self,
        _mr_iid: u64,
        _body: &str,
        _file_path: Option<&str>,
        _new_line: Option<usize>,
    ) -> Result<()> {
        // TODO: implement post_comment
        Ok(())
    }

    async fn approve_mr(&self, _mr_iid: u64) -> Result<()> {
        // TODO: implement approve_mr
        Ok(())
    }
}
