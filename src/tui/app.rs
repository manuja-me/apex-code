#![allow(dead_code)]
use std::path::{Path, PathBuf};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::agent::engine::AgentEvent;
use crate::config::ApexConfig;
use crate::tools::git_ops::git_diff;

#[derive(Debug, Clone)]
pub enum TuiMessageKind {
    User,
    Assistant,
    ToolCall { name: String, args: String },
    ToolResult { name: String, duration_ms: u128 },
    System,
    Error,
}

#[derive(Debug, Clone)]
pub struct TuiMessage {
    pub kind: TuiMessageKind,
    pub content: String,
}

pub struct App {
    pub workspace: PathBuf,
    pub config: ApexConfig,
    pub input: String,
    pub cursor_pos: usize,
    pub messages: Vec<TuiMessage>,
    pub active_model: String,
    pub is_running: bool,
    pub scroll: usize,
    pub sidebar_tab: usize,
    pub token_count: usize,
    pub cost: f64,
    pub git_branch: String,
    pub should_quit: bool,
    pub status_text: String,
    // Prompt history
    pub prompt_history: Vec<String>,
    pub history_idx: Option<usize>,
    pub saved_draft: String,
}

impl App {
    pub fn new(workspace: impl AsRef<Path>, config: ApexConfig) -> Self {
        let branch = Self::detect_git_branch(workspace.as_ref());
        let primary_model = config.models.primary.clone();

        let mut app = Self {
            workspace: workspace.as_ref().to_path_buf(),
            config,
            input: String::new(),
            cursor_pos: 0,
            messages: Vec::new(),
            active_model: primary_model,
            is_running: false,
            scroll: 0,
            sidebar_tab: 0,
            token_count: 0,
            cost: 0.0,
            git_branch: branch,
            should_quit: false,
            status_text: "NORMAL // READY".to_string(),
            prompt_history: Vec::new(),
            history_idx: None,
            saved_draft: String::new(),
        };

        app.messages.push(TuiMessage {
            kind: TuiMessageKind::System,
            content: "APEX // HIGH-PERFORMANCE CODING ENGINE ONLINE. Type /help for slash commands.".to_string(),
        });

        app
    }

    fn detect_git_branch(workspace: &Path) -> String {
        if let Ok(output) = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(workspace)
            .output()
        {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    return format!("{}*", name);
                }
            }
        }
        "main*".to_string()
    }

    /// Process slash commands (/help, /clear, /model, /diff, /status, /quit)
    pub async fn handle_slash_command(&mut self, cmd_line: &str) -> bool {
        let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "/help" => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::System,
                    content: "=== APEX SLASH COMMANDS & KEYBINDINGS ===\n\
                             /help           - Show this help reference\n\
                             /clear          - Clear conversation stream\n\
                             /model <id>     - Switch active model on the fly\n\
                             /diff           - Run and display current git diff\n\
                             /status         - Display token telemetry & branch status\n\
                             /quit           - Exit Apex\n\n\
                             [KEYBINDINGS]\n\
                             Enter           - Submit prompt\n\
                             Tab             - Cycle sidebar panel\n\
                             Up / Down       - Browse prompt history\n\
                             PageUp / PageDn - Scroll activity stream\n\
                             Home / End      - Jump to line start / end\n\
                             Delete          - Delete character forward\n\
                             Ctrl+W          - Delete word backward\n\
                             Ctrl+U          - Clear line\n\
                             Esc             - Cancel / Exit".to_string(),
                });
                true
            }
            "/clear" => {
                self.messages.clear();
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::System,
                    content: "Conversation history cleared.".to_string(),
                });
                self.scroll = 0;
                true
            }
            "/model" => {
                if parts.len() > 1 {
                    let new_model = parts[1].to_string();
                    self.active_model = new_model.clone();
                    self.messages.push(TuiMessage {
                        kind: TuiMessageKind::System,
                        content: format!("Active model switched to: {}", new_model),
                    });
                } else {
                    self.messages.push(TuiMessage {
                        kind: TuiMessageKind::System,
                        content: format!("Current active model: {}\nUsage: /model <model_id> (e.g. /model deepseek/deepseek-r1:free)", self.active_model),
                    });
                }
                true
            }
            "/diff" => {
                let diff_output = match git_diff(&self.workspace, None).await {
                    Ok(out) => out,
                    Err(err) => format!("Error running git diff: {}", err),
                };
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::System,
                    content: format!("=== CURRENT GIT DIFF ===\n{}", diff_output),
                });
                true
            }
            "/status" => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::System,
                    content: format!(
                        "Active Model:    {}\n\
                         Session Tokens:  {}\n\
                         Estimated Cost:  ${:.4}\n\
                         Git Branch:      {}\n\
                         Workspace:       {}",
                        self.active_model, self.token_count, self.cost, self.git_branch, self.workspace.display()
                    ),
                });
                true
            }
            "/quit" | "/exit" => {
                self.should_quit = true;
                true
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle Ctrl combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('a') => {
                    self.cursor_pos = 0;
                    return;
                }
                KeyCode::Char('e') => {
                    self.cursor_pos = self.input.len();
                    return;
                }
                KeyCode::Char('u') => {
                    self.input.clear();
                    self.cursor_pos = 0;
                    return;
                }
                KeyCode::Char('w') => {
                    self.delete_word_backward();
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => {
                if self.is_running {
                    self.status_text = "INTERRUPTED".to_string();
                    self.is_running = false;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Tab => {
                self.sidebar_tab = (self.sidebar_tab + 1) % 3;
            }
            // Stream scrolling with PageUp / PageDown
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(6);
            }
            KeyCode::PageDown => {
                self.scroll += 6;
            }
            // Prompt History Navigation with Up / Down
            KeyCode::Up => {
                self.history_prev();
            }
            KeyCode::Down => {
                self.history_next();
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                // Reset history browsing when actively modifying input
                self.history_idx = None;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 && !self.input.is_empty() {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                    self.history_idx = None;
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            _ => {}
        }
    }

    fn delete_word_backward(&mut self) {
        if self.cursor_pos == 0 || self.input.is_empty() {
            return;
        }

        let before = &self.input[..self.cursor_pos];
        let trimmed = before.trim_end();

        let word_start = trimmed.rfind(' ').map(|idx| idx + 1).unwrap_or(0);
        let chars_to_remove = self.cursor_pos - word_start;

        for _ in 0..chars_to_remove {
            if word_start < self.input.len() {
                self.input.remove(word_start);
            }
        }
        self.cursor_pos = word_start;
        self.history_idx = None;
    }

    fn history_prev(&mut self) {
        if self.prompt_history.is_empty() {
            return;
        }

        if self.history_idx.is_none() {
            self.saved_draft = self.input.clone();
            let new_idx = self.prompt_history.len() - 1;
            self.history_idx = Some(new_idx);
            self.input = self.prompt_history[new_idx].clone();
            self.cursor_pos = self.input.len();
        } else if let Some(idx) = self.history_idx {
            if idx > 0 {
                let new_idx = idx - 1;
                self.history_idx = Some(new_idx);
                self.input = self.prompt_history[new_idx].clone();
                self.cursor_pos = self.input.len();
            }
        }
    }

    fn history_next(&mut self) {
        if let Some(idx) = self.history_idx {
            if idx + 1 < self.prompt_history.len() {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input = self.prompt_history[new_idx].clone();
                self.cursor_pos = self.input.len();
            } else {
                self.history_idx = None;
                self.input = self.saved_draft.clone();
                self.cursor_pos = self.input.len();
            }
        }
    }

    pub fn append_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::ModelSelected(m) => {
                self.active_model = m;
            }
            AgentEvent::AssistantMessage(text) => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::Assistant,
                    content: text,
                });
            }
            AgentEvent::ToolExecuting { name, args, .. } => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::ToolCall { name, args: args.clone() },
                    content: format!("EXECUTING TOOL: {}", args),
                });
            }
            AgentEvent::ToolCompleted { name, output, duration_ms, .. } => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::ToolResult { name, duration_ms },
                    content: output,
                });
            }
            AgentEvent::UsageUpdate { session_tokens, estimated_cost, .. } => {
                self.token_count = session_tokens;
                self.cost = estimated_cost;
            }
            AgentEvent::Error(err) => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::Error,
                    content: err,
                });
            }
            AgentEvent::StepDone => {
                self.scroll = self.messages.len().saturating_sub(6);
            }
        }
    }
}
