use std::path::Path;
use anyhow::Result;
use crate::tools::terminal::run_command;

pub async fn git_status(workspace: &Path) -> Result<String> {
    let out = run_command(workspace, "git status --short --branch", 10).await?;
    Ok(out)
}

pub async fn git_diff(workspace: &Path, file_path: Option<&str>) -> Result<String> {
    let cmd = match file_path {
        Some(path) => format!("git diff -- {}", path),
        None => "git diff".to_string(),
    };
    let out = run_command(workspace, &cmd, 10).await?;
    if out.trim().is_empty() || out.contains("Exit Code: 0\n") && !out.contains("--- STDOUT ---") {
        Ok("No unstaged git diffs found.".to_string())
    } else {
        Ok(out)
    }
}
