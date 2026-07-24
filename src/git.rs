use std::process::Command;
use anyhow::{Context, Result};

/// Returns the current git branch name trimmed.
/// Returns an error if not in a git repository or if in detached HEAD state.
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git error or not in a git repository: {}", stderr.trim());
    }

    let branch = String::from_utf8(output.stdout)
        .context("Git branch output is not valid UTF-8")?
        .trim()
        .to_string();

    if branch.is_empty() {
        anyhow::bail!("No current branch detected (possibly detached HEAD)");
    }

    Ok(branch)
}

/// Returns the URL for the specified git remote trimmed.
///
/// Default usage will pass `"origin"`.
pub fn get_remote_url(remote: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .with_context(|| format!("Failed to execute git command for remote '{}'", remote))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get URL for remote '{}': {}", remote, stderr.trim());
    }

    let url = String::from_utf8(output.stdout)
        .context("Git remote URL output is not valid UTF-8")?
        .trim()
        .to_string();

    if url.is_empty() {
        anyhow::bail!("Remote URL for '{}' is empty", remote);
    }

    Ok(url)
}

/// Parses a git remote URL into `(project_path, base_url)`.
///
/// Supports HTTPS, HTTP, SSH (SCP-style and `ssh://`), with or without `.git` extension,
/// and self-hosted instances.
pub fn parse_remote_url(url: &str) -> Result<(String, String)> {
    let clean_url = url.trim();
    if clean_url.is_empty() {
        anyhow::bail!("Remote URL is empty");
    }

    let clean_url = clean_url.strip_suffix(".git").unwrap_or(clean_url);

    if clean_url.starts_with("https://") || clean_url.starts_with("http://") {
        let (scheme, rest) = clean_url
            .split_once("://")
            .ok_or_else(|| anyhow::anyhow!("Invalid URL structure: '{}'", url))?;

        let (host_part, path_part) = rest.split_once('/').ok_or_else(|| {
            anyhow::anyhow!("Invalid remote URL (missing path): '{}'", url)
        })?;

        let host_without_user = host_part
            .rsplit_once('@')
            .map(|(_, h)| h)
            .unwrap_or(host_part);

        let path = path_part.trim_matches('/');
        if path.is_empty() {
            anyhow::bail!("Invalid remote URL (empty project path): '{}'", url);
        }

        let base_url = format!("{}://{}", scheme, host_without_user);
        let project_path = path.to_string();
        Ok((project_path, base_url))
    } else if clean_url.starts_with("ssh://") {
        let rest = &clean_url["ssh://".len()..];
        let (host_part, path_part) = rest.split_once('/').ok_or_else(|| {
            anyhow::anyhow!("Invalid SSH remote URL (missing path): '{}'", url)
        })?;

        let host_without_user = host_part
            .rsplit_once('@')
            .map(|(_, h)| h)
            .unwrap_or(host_part);

        let host_domain = host_without_user.split(':').next().unwrap_or(host_without_user);
        if host_domain.is_empty() {
            anyhow::bail!("Invalid SSH remote URL (empty host): '{}'", url);
        }

        let path = path_part.trim_matches('/');
        if path.is_empty() {
            anyhow::bail!("Invalid SSH remote URL (empty project path): '{}'", url);
        }

        let base_url = format!("https://{}", host_domain);
        let project_path = path.to_string();
        Ok((project_path, base_url))
    } else if let Some((host_part, path_part)) = clean_url.split_once(':') {
        let host_without_user = host_part
            .rsplit_once('@')
            .map(|(_, h)| h)
            .unwrap_or(host_part);

        let host_domain = host_without_user.trim();
        if host_domain.is_empty() {
            anyhow::bail!("Invalid SSH remote URL (empty host): '{}'", url);
        }

        let path = path_part.trim_matches('/');
        if path.is_empty() {
            anyhow::bail!("Invalid SSH remote URL (empty project path): '{}'", url);
        }

        let base_url = format!("https://{}", host_domain);
        let project_path = path.to_string();
        Ok((project_path, base_url))
    } else {
        anyhow::bail!("Unrecognized git remote URL format: '{}'", url);
    }
}

/// Detects the project path and base URL from the `origin` remote URL.
/// Returns `(project_path, base_url)`.
pub fn detect_project_from_remote() -> Result<(String, String)> {
    let url = get_remote_url("origin")?;
    parse_remote_url(&url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_with_git_extension() {
        let url = "https://gitlab.com/group/subgroup/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/subgroup/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_ssh_with_git_extension() {
        let url = "git@gitlab.com:group/subgroup/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/subgroup/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_https_without_git_extension() {
        let url = "https://gitlab.com/group/project";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_ssh_without_git_extension() {
        let url = "git@gitlab.com:group/project";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_self_hosted_https() {
        let url = "https://gitlab.mycompany.com/team/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "team/project");
        assert_eq!(base_url, "https://gitlab.mycompany.com");
    }

    #[test]
    fn test_parse_self_hosted_ssh() {
        let url = "git@gitlab.mycompany.com:team/subteam/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "team/subteam/project");
        assert_eq!(base_url, "https://gitlab.mycompany.com");
    }

    #[test]
    fn test_parse_ssh_scheme_url() {
        let url = "ssh://git@gitlab.com/group/subgroup/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/subgroup/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_https_with_userinfo() {
        let url = "https://oauth2:secret_token@gitlab.com/group/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "group/project");
        assert_eq!(base_url, "https://gitlab.com");
    }

    #[test]
    fn test_parse_http_with_port() {
        let url = "http://gitlab.local:8080/team/project.git";
        let (project_path, base_url) = parse_remote_url(url).unwrap();
        assert_eq!(project_path, "team/project");
        assert_eq!(base_url, "http://gitlab.local:8080");
    }

    #[test]
    fn test_parse_invalid_urls() {
        assert!(parse_remote_url("").is_err());
        assert!(parse_remote_url("invalid_url").is_err());
        assert!(parse_remote_url("https://gitlab.com").is_err());
        assert!(parse_remote_url("git@gitlab.com:").is_err());
    }
}
