use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MergeRequest {
    iid: u64,
    title: String,
    web_url: String,
}

async fn find_mr(
    project: &str,
    branch: &str,
) -> Result<Vec<MergeRequest>, reqwest::Error> {

    let project = urlencoding::encode(project);

    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/merge_requests?source_branch={}",
        project,
        branch
    );

    let response = reqwest::get(url).await?.json::<Vec<MergeRequest>>().await?;

    Ok(response)
}

fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        return Err("Not inside a git repository".to_string());
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8: {}", e))?
        .trim()
        .to_string();

    if branch.is_empty() {
        return Err("No branch detected (possibly detached HEAD)".to_string());
    }

    Ok(branch)
}

#[tokio::main]
async fn main() {
    let project = "gitlab-org/gitlab";
    let branch = match get_current_branch() {
        Ok(branch) => branch,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    println!("Branch: {}", branch);

    match find_mr(project, &branch).await {
        Ok(mrs) => {
             if mrs.is_empty() {
                println!("No merge request found for {}", branch);
             } else {
                for mr in mrs {
                    println!("#{} :- {}", mr.iid, mr.title);
                    println!("url: {}", mr.web_url);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch MR: {}", err);
        }
    }
}
