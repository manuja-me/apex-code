use std::path::Path;
use anyhow::Result;
use ignore::WalkBuilder;

pub fn ripgrep_search(workspace: &Path, pattern: &str, subpath: Option<&str>, case_insensitive: bool) -> Result<String> {
    let search_root = match subpath {
        Some(sub) => workspace.join(sub),
        None => workspace.to_path_buf(),
    };

    let mut regex_builder = regex::RegexBuilder::new(pattern);
    regex_builder.case_insensitive(case_insensitive);
    let re = regex_builder.build()?;

    let mut matches = Vec::new();
    let max_matches = 60;

    let walker = WalkBuilder::new(&search_root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();
        let relative_path = path.strip_prefix(workspace).unwrap_or(path);

        if let Ok(content) = std::fs::read_to_string(path) {
            for (line_idx, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    matches.push(format!(
                        "{}:{}: {}",
                        relative_path.display(),
                        line_idx + 1,
                        line.trim_end()
                    ));

                    if matches.len() >= max_matches {
                        matches.push(format!("... (Results truncated at {} matches)", max_matches));
                        return Ok(matches.join("\n"));
                    }
                }
            }
        }
    }

    if matches.is_empty() {
        Ok(format!("No matches found for pattern: '{}'", pattern))
    } else {
        Ok(matches.join("\n"))
    }
}

pub fn find_files(workspace: &Path, query: &str) -> Result<String> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    let max_results = 50;

    let walker = WalkBuilder::new(workspace)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let rel_path = path.strip_prefix(workspace).unwrap_or(path).to_string_lossy();

        if rel_path.to_lowercase().contains(&query_lower) {
            let kind = if entry.file_type().map_or(false, |f| f.is_dir()) { "DIR " } else { "FILE" };
            results.push(format!("[{}] {}", kind, rel_path));
            if results.len() >= max_results {
                results.push(format!("... (Truncated at {} matches)", max_results));
                break;
            }
        }
    }

    if results.is_empty() {
        Ok(format!("No files or directories matching '{}'", query))
    } else {
        Ok(results.join("\n"))
    }
}
