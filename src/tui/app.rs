#![allow(dead_code)]
use std::path::{Path, PathBuf};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::agent::engine::AgentEvent;
use crate::config::ApexConfig;

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
        };

        app.messages.push(TuiMessage {
            kind: TuiMessageKind::System,
            content: "APEX // HIGH-PERFORMANCE CODING ENGINE ONLINE. Type a prompt or command.".to_string(),
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
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
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Down => {
                self.scroll += 1;
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 && !self.input.is_empty() {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
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
            AgentEvent::Error(err) => {
                self.messages.push(TuiMessage {
                    kind: TuiMessageKind::Error,
                    content: err,
                });
            }
            AgentEvent::StepDone => {
                self.scroll = self.messages.len().saturating_sub(5);
            }
        }
    }
}
