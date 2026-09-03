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
    StepDone,
    Error(String),
}

pub struct AgentEngine {
    pub workspace: PathBuf,
    pub config: ApexConfig,
    pub client: OpenRouterClient,
    pub history: Vec<ChatMessage>,
    pub tools: Vec<ToolDefinition>,
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
        };

        engine.init_system_prompt(&workspace);
        Ok(engine)
    }

    fn init_system_prompt(&mut self, workspace: &Path) {
        let workspace_str = workspace.display().to_string();
        let system_prompt = format!(
            "You are APEX, an elite, high-performance autonomous coding agent operating directly in the user's terminal.\n\
            Current Workspace: {}\n\n\
            CORE OPERATING PRINCIPLES:\n\
            1. Precision & Verification: Always inspect files and locate symbols before modifying. After edits, run build/check/tests to verify.\n\
            2. Minimal Disruption: Use surgical edits (`edit_file`) whenever possible rather than rewriting whole files.\n\
            3. Tool Autonomy: You have full access to native ripgrep, file viewing, file writing/editing, terminal commands, and git diffs.\n\
            4. Concise & Decisive: Avoid long conversational fluff. State actions clearly, execute tools proactively, and report verified results.",
            workspace_str
        );

        self.history.push(ChatMessage::system(system_prompt));
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
            let (model, content, tool_calls) = match self.client.chat_with_fallback(&self.history, Some(&self.tools)).await {
                Ok(res) => res,
                Err(err) => {
                    on_event(AgentEvent::Error(err.to_string()));
                    return Err(err);
                }
            };

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
