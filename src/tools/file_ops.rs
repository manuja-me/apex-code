use std::path::Path;
use anyhow::{bail, Context, Result};

pub fn view_file(workspace: &Path, rel_path: &str, start_line: Option<usize>, end_line: Option<usize>) -> Result<String> {
    let file_path = workspace.join(rel_path);
    if !file_path.exists() {
        bail!("File not found: {}", rel_path);
    }

    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {}", rel_path))?;

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = start_line.unwrap_or(1).max(1);
    let end = end_line.unwrap_or(total_lines).min(total_lines);

    if start > total_lines {
        return Ok(format!("File {} has {} lines. Requested start line {} is out of range.", rel_path, total_lines, start));
    }

    let mut output = Vec::new();
    output.push(format!("--- {} (Lines {}-{} of {}) ---", rel_path, start, end, total_lines));

    for idx in (start - 1)..end {
        if let Some(line) = lines.get(idx) {
            output.push(format!("{:4} | {}", idx + 1, line));
        }
    }

    Ok(output.join("\n"))
}

pub fn write_file(workspace: &Path, rel_path: &str, content: &str, overwrite: bool) -> Result<String> {
    let file_path = workspace.join(rel_path);

    if file_path.exists() && !overwrite {
        bail!("File already exists: {}. Pass overwrite=true to replace.", rel_path);
    }

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directories for: {}", rel_path))?;
    }

    std::fs::write(&file_path, content)
        .with_context(|| format!("Failed to write file: {}", rel_path))?;

    Ok(format!("Successfully wrote {} ({} bytes)", rel_path, content.len()))
}

pub fn edit_file(
    workspace: &Path,
    rel_path: &str,
    target_content: &str,
    replacement_content: &str,
) -> Result<String> {
    let file_path = workspace.join(rel_path);
    if !file_path.exists() {
        bail!("File not found: {}", rel_path);
    }

    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {}", rel_path))?;

    // 1. Normalize line endings for reliable matching
    let normalized_content = content.replace("\r\n", "\n");
    let normalized_target = target_content.replace("\r\n", "\n");
    let normalized_replacement = replacement_content.replace("\r\n", "\n");

    let match_count = normalized_content.matches(&normalized_target).count();

    if match_count == 1 {
        let new_content = normalized_content.replacen(&normalized_target, &normalized_replacement, 1);
        let final_content = if content.contains("\r\n") {
            new_content.replace("\n", "\r\n")
        } else {
            new_content
        };
        std::fs::write(&file_path, final_content)
            .with_context(|| format!("Failed to write modified file: {}", rel_path))?;
        return Ok(format!("Successfully applied edit to {}", rel_path));
    }

    if match_count > 1 {
        bail!(
            "target_content matched {} times in {}. Please include more surrounding context to make it unique.",
            match_count,
            rel_path
        );
    }

    // 2. Whitespace-tolerant line matching fallback if exact match fails
    let file_lines: Vec<&str> = normalized_content.lines().collect();
    let target_lines: Vec<&str> = normalized_target.lines().collect();

    if !target_lines.is_empty() && file_lines.len() >= target_lines.len() {
        let mut matched_indices = Vec::new();

        for i in 0..=(file_lines.len() - target_lines.len()) {
            let window = &file_lines[i..i + target_lines.len()];
            let mut matches = true;

            for (file_l, target_l) in window.iter().zip(target_lines.iter()) {
                if file_l.trim() != target_l.trim() {
                    matches = false;
                    break;
                }
            }

            if matches {
                matched_indices.push(i);
            }
        }

        if matched_indices.len() == 1 {
            let start_idx = matched_indices[0];
            let end_idx = start_idx + target_lines.len();

            let mut new_lines = Vec::new();
            new_lines.extend_from_slice(&file_lines[..start_idx]);
            new_lines.push(&normalized_replacement);
            new_lines.extend_from_slice(&file_lines[end_idx..]);

            let joined = new_lines.join("\n");
            let final_content = if content.contains("\r\n") {
                joined.replace("\n", "\r\n")
            } else {
                joined
            };

            std::fs::write(&file_path, final_content)
                .with_context(|| format!("Failed to write modified file: {}", rel_path))?;

            return Ok(format!("Successfully applied whitespace-tolerant edit to {}", rel_path));
        } else if matched_indices.len() > 1 {
            bail!(
                "target_content matched {} distinct locations in {}. Please include more surrounding code.",
                matched_indices.len(),
                rel_path
            );
        }
    }

    bail!("target_content not found in {}. Please verify line numbers and exact content.", rel_path);
}
