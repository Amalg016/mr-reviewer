use std::process::Command;

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

fn main() {
    let project = "gitlab-org/gitlab";
    let branch = match get_current_branch() {
        Ok(branch) => branch,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

println!("Branch: {}", branch);

}
