use std::path::Path;
use std::process::Stdio;
use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_command(workspace: &Path, command_line: &str, timeout_secs: u64) -> Result<String> {
    #[cfg(target_os = "windows")]
    let (program, args) = ("powershell.exe", vec!["-NoProfile", "-Command", command_line]);

    #[cfg(not(target_os = "windows"))]
    let (program, args) = ("sh", vec!["-c", command_line]);

    let mut cmd = Command::new(program);
    cmd.args(&args)
        .current_dir(workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn().with_context(|| format!("Failed to spawn command: {}", command_line))?;

    let timeout_duration = Duration::from_secs(timeout_secs);
    let output = match timeout(timeout_duration, child.wait_with_output()).await {
        Ok(res) => res.context("Command failed while executing")?,
        Err(_) => {
            return Ok(format!("Command timed out after {} seconds: {}", timeout_secs, command_line));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    let max_len = 10000;
    let truncated_stdout = if stdout.len() > max_len {
        format!("{}...\n(Output truncated at {} chars)", &stdout[..max_len], max_len)
    } else {
        stdout.to_string()
    };

    let mut response = Vec::new();
    response.push(format!("Exit Code: {}", code));
    if !truncated_stdout.trim().is_empty() {
        response.push("--- STDOUT ---".to_string());
        response.push(truncated_stdout);
    }
    if !stderr.trim().is_empty() {
        response.push("--- STDERR ---".to_string());
        response.push(stderr.to_string());
    }

    Ok(response.join("\n"))
}
