use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;
use crate::agent::types::*;
use crate::config::ApexConfig;
use crate::providers::OpenRouterClient;
use crate::tools::{execute_tool, get_tool_definitions};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AgentEvent {
    ModelSelected(String),
    AssistantMessage(String),
    ToolExecuting { id: String, name: String, args: String },
    ToolCompleted { id: String, name: String, output: String, duration_ms: u128 },
    UsageUpdate { prompt_tokens: usize, completion_tokens: usize, session_tokens: usize, estimated_cost: f64 },
    StepDone,
    Error(String),
}

pub struct AgentEngine {
    pub workspace: PathBuf,
    pub config: ApexConfig,
    pub client: OpenRouterClient,
    pub history: Vec<ChatMessage>,
    pub tools: Vec<ToolDefinition>,
    pub session_tokens: usize,
}

impl AgentEngine {
    pub fn new(workspace: impl AsRef<Path>, config: ApexConfig) -> Result<Self> {
        let workspace = workspace.as_ref().to_path_buf();
        let client = OpenRouterClient::new(config.clone())?;
        let tools = get_tool_definitions();

        let mut engine = Self {
            workspace: workspace.clone(),
            config,
            client,
            history: Vec::new(),
            tools,
            session_tokens: 0,
        };

        engine.init_system_prompt(&workspace);
        Ok(engine)
    }

    fn detect_project_context(workspace: &Path) -> String {
        let mut details: Vec<String> = Vec::new();

        if workspace.join("Cargo.toml").exists() {
            details.push("Project Type: Rust (Cargo)".to_string());
        } else if workspace.join("package.json").exists() {
            details.push("Project Type: Node.js / TypeScript".to_string());
        } else if workspace.join("pyproject.toml").exists() || workspace.join("requirements.txt").exists() {
            details.push("Project Type: Python".to_string());
        } else if workspace.join("go.mod").exists() {
            details.push("Project Type: Go".to_string());
        }

        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--short", "--branch"])
            .current_dir(workspace)
            .output()
        {
            if output.status.success() {
                let git_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !git_info.is_empty() {
                    details.push(format!("Git Workspace Status:\n{}", git_info));
                }
            }
        }

        details.join("\n")
    }

    fn init_system_prompt(&mut self, workspace: &Path) {
        let workspace_str = workspace.display().to_string();
        let context_str = Self::detect_project_context(workspace);

        let system_prompt = format!(
            "You are APEX, an elite, high-performance autonomous coding agent operating directly in the user's terminal.\n\
            Current Workspace: {}\n\
            {}\n\n\
            CORE OPERATING PRINCIPLES:\n\
            1. Precision & Verification: Always inspect files and locate symbols before modifying. After edits, run build/check/tests to verify.\n\
            2. Minimal Disruption: Use surgical edits (`edit_file`) whenever possible rather than rewriting whole files.\n\
            3. Tool Autonomy: You have full access to native ripgrep, file viewing, file writing/editing, terminal commands, and git diffs.\n\
            4. Concise & Decisive: Avoid long conversational fluff. State actions clearly, execute tools proactively, and report verified results.",
            workspace_str, context_str
        );

        self.history.push(ChatMessage::system(system_prompt));
    }

    /// Reset conversation history while preserving system instructions
    #[allow(dead_code)]
    pub fn reset_history(&mut self) {
        let system_msg = self.history.first().cloned();
        self.history.clear();
        if let Some(sys) = system_msg {
            self.history.push(sys);
        }
    }

    /// Run an agent task to completion, invoking the event callback at each milestone
    pub async fn run<F>(&mut self, prompt: &str, mut on_event: F) -> Result<()>
    where
        F: FnMut(AgentEvent),
    {
        self.history.push(ChatMessage::user(prompt));
        let max_steps = self.config.agent.max_steps;

        for _step_idx in 0..max_steps {
            // Request completion from OpenRouter with fallback
            let (model, content, tool_calls, prompt_tokens, completion_tokens) = match self.client.chat_with_fallback(&self.history, Some(&self.tools)).await {
                Ok(res) => res,
                Err(err) => {
                    on_event(AgentEvent::Error(err.to_string()));
                    return Err(err);
                }
            };

            self.session_tokens += prompt_tokens + completion_tokens;
            // Free tier models have $0.00 cost; if paid, calculate estimate
            let cost = if model.contains(":free") {
                0.0
            } else {
                (self.session_tokens as f64 / 1_000_000.0) * 0.50
            };

            on_event(AgentEvent::UsageUpdate {
                prompt_tokens,
                completion_tokens,
                session_tokens: self.session_tokens,
                estimated_cost: cost,
            });

            on_event(AgentEvent::ModelSelected(model));

            // Record assistant message
            self.history.push(ChatMessage::assistant(content.clone(), tool_calls.clone()));

            if let Some(ref text) = content {
                if !text.trim().is_empty() {
                    on_event(AgentEvent::AssistantMessage(text.clone()));
                }
            }

            // If there are tool calls, execute them
            if let Some(ref calls) = tool_calls {
                if calls.is_empty() {
                    break;
                }

                for call in calls {
                    on_event(AgentEvent::ToolExecuting {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        args: call.function.arguments.clone(),
                    });

                    let start_time = Instant::now();
                    let output = match execute_tool(&self.workspace, &call.function.name, &call.function.arguments).await {
                        Ok(res) => res,
                        Err(err) => format!("Error executing tool '{}': {}", call.function.name, err),
                    };
                    let duration_ms = start_time.elapsed().as_millis();

                    on_event(AgentEvent::ToolCompleted {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        output: output.clone(),
                        duration_ms,
                    });

                    self.history.push(ChatMessage::tool(output, call.id.clone()));
                }

                on_event(AgentEvent::StepDone);
            } else {
                // No more tool calls; assistant finished response
                break;
            }
        }

        Ok(())
    }
}
