pub mod search;
pub mod file_ops;
pub mod terminal;
pub mod git_ops;

use std::path::Path;
use anyhow::{bail, Result};
use serde_json::json;
use crate::agent::types::ToolDefinition;

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition::function(
            "ripgrep",
            "Search for text/regex patterns across files in the workspace (ignores gitignored files)",
            json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regular expression or text pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory to restrict the search to"
                    },
                    "case_insensitive": {
                        "type": "boolean",
                        "description": "Whether to perform case-insensitive matching (default: false)"
                    }
                },
                "required": ["pattern"]
            }),
        ),
        ToolDefinition::function(
            "find_files",
            "Find files and directories matching a query substring or extension across the workspace",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Substring or extension to search for in filenames (e.g. 'auth', '.rs', 'config')"
                    }
                },
                "required": ["query"]
            }),
        ),
        ToolDefinition::function(
            "view_file",
            "View file content with line numbers and optional line range window",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path of the file to read"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Optional starting line number (1-indexed)"
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "Optional ending line number (inclusive)"
                    }
                },
                "required": ["path"]
            }),
        ),
        ToolDefinition::function(
            "write_file",
            "Create a new file or completely overwrite an existing file with the provided content",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path of the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The exact full text content to write"
                    },
                    "overwrite": {
                        "type": "boolean",
                        "description": "Must be set to true if file already exists"
                    }
                },
                "required": ["path", "content"]
            }),
        ),
        ToolDefinition::function(
            "edit_file",
            "Make a surgical edit by replacing an exact, unique block of target content with replacement content",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path of the file to edit"
                    },
                    "target_content": {
                        "type": "string",
                        "description": "The exact existing lines to replace (must match uniquely)"
                    },
                    "replacement_content": {
                        "type": "string",
                        "description": "The new replacement lines"
                    }
                },
                "required": ["path", "target_content", "replacement_content"]
            }),
        ),
        ToolDefinition::function(
            "run_command",
            "Execute a shell command in the project workspace with timeout and output capture",
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command line to run (e.g. 'cargo check', 'npm test', 'ls')"
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Optional timeout in seconds (default: 30)"
                    }
                },
                "required": ["command"]
            }),
        ),
        ToolDefinition::function(
            "git_status",
            "Check current git branch, staged files, unstaged modifications, and untracked files",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        ToolDefinition::function(
            "git_diff",
            "Inspect current unstaged diffs across the repo or for a specific file",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Optional specific file path to inspect diff for"
                    }
                }
            }),
        ),
    ]
}

pub async fn execute_tool(workspace: &Path, name: &str, arguments_json: &str) -> Result<String> {
    let args: serde_json::Value = serde_json::from_str(arguments_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    match name {
        "ripgrep" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let subpath = args.get("path").and_then(|v| v.as_str());
            let case_insensitive = args.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
            search::ripgrep_search(workspace, pattern, subpath, case_insensitive)
        }
        "find_files" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            search::find_files(workspace, query)
        }
        "view_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let start = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
            let end = args.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);
            file_ops::view_file(workspace, path, start, end)
        }
        "write_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let overwrite = args.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(true);
            file_ops::write_file(workspace, path, content, overwrite)
        }
        "edit_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let target = args.get("target_content").and_then(|v| v.as_str()).unwrap_or("");
            let replacement = args.get("replacement_content").and_then(|v| v.as_str()).unwrap_or("");
            file_ops::edit_file(workspace, path, target, replacement)
        }
        "run_command" => {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let timeout = args.get("timeout_seconds").and_then(|v| v.as_u64()).unwrap_or(30);
            terminal::run_command(workspace, cmd, timeout).await
        }
        "git_status" => {
            git_ops::git_status(workspace).await
        }
        "git_diff" => {
            let path = args.get("path").and_then(|v| v.as_str());
            git_ops::git_diff(workspace, path).await
        }
        _ => bail!("Unknown tool: {}", name),
    }
}
