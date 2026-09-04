use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use crate::agent::types::*;
use crate::config::ApexConfig;

pub struct OmniRouteClient {
    client: Client,
    api_key: String,
    base_url: String,
    config: ApexConfig,
}

impl OmniRouteClient {
    pub fn new(config: ApexConfig) -> Result<Self> {
        let api_key = config.get_api_key().unwrap_or_default();
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: config.provider.base_url.clone(),
            config,
        })
    }

    /// Complete a chat step with automatic model fallback on 429 / queue saturation
    /// Returns: (selected_model, content, tool_calls, prompt_tokens, completion_tokens)
    pub async fn chat_with_fallback(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<(String, Option<String>, Option<Vec<ToolCall>>, usize, usize)> {
        let is_local = self.base_url.contains("localhost") || self.base_url.contains("127.0.0.1");
        if self.api_key.is_empty() && !is_local {
            bail!(
                "No API key found for gateway '{}'!\n\
                 Set your key with: set OMNIROUTE_API_KEY=your_key or in .apex/config.toml",
                self.base_url
            );
        }

        let mut models_to_try = vec![self.config.models.primary.clone()];
        for m in &self.config.models.fallback_pool {
            if !models_to_try.contains(m) {
                models_to_try.push(m.clone());
            }
        }

        let mut last_err = String::new();

        for model in &models_to_try {
            match self.send_chat_request(model, messages, tools).await {
                Ok(response) => {
                    return Ok((model.clone(), response.0, response.1, response.2, response.3));
                }
                Err(err) => {
                    let err_str = err.to_string();
                    last_err = err_str.clone();

                    // Check for rate limit or queue overload
                    if err_str.contains("429") || err_str.contains("rate limit") || err_str.contains("busy") || err_str.contains("overloaded") {
                        eprintln!("\x1b[33m[!] Model '{}' rate-limited or busy. Auto-routing to next fallback model in OmniRoute pool...\x1b[0m", model);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        continue;
                    } else {
                        // Other error, also try fallback if auto_fallback is true
                        if self.config.models.auto_fallback {
                            eprintln!("\x1b[33m[!] Model '{}' failed: {}. Trying fallback...\x1b[0m", model, err);
                            continue;
                        } else {
                            bail!(err);
                        }
                    }
                }
            }
        }

        bail!("All models in fallback pool failed. Last error: {}", last_err)
    }

    async fn send_chat_request(
        &self,
        model: &str,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<(Option<String>, Option<Vec<ToolCall>>, usize, usize)> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let request_payload = ChatCompletionRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            stream: Some(false),
            temperature: Some(self.config.agent.temperature),
            max_tokens: Some(self.config.agent.max_tokens),
        };

        let mut request = self.client.post(&url).json(&request_payload);

        let token = if !self.api_key.is_empty() {
            &self.api_key
        } else {
            "omniroute"
        };
        request = request.header("Authorization", format!("Bearer {}", token));

        if self.base_url.contains("openrouter.ai") {
            request = request
                .header("HTTP-Referer", "https://github.com/manuja-me/apex-code")
                .header("X-Title", "Apex Code Agent");
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to connect to OmniRoute gateway at {}. Ensure OmniRoute is running.", url))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            bail!("OmniRoute gateway error (HTTP {}): {}", status, error_text);
        }

        let resp_json: Value = response.json().await
            .with_context(|| format!("Failed to parse JSON response from OmniRoute gateway at {}", url))?;

        let choice = resp_json.get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .context("No choices returned in API response")?;

        let message = choice.get("message").context("Missing message in choice")?;

        let content = message.get("content").and_then(|c| c.as_str()).map(|s| s.to_string());

        let tool_calls = if let Some(tc_array) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
            let mut calls = Vec::new();
            for tc in tc_array {
                let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("call_0").to_string();
                let call_type = tc.get("type").and_then(|v| v.as_str()).unwrap_or("function").to_string();
                let fn_val = tc.get("function").context("Missing function in tool_call")?;
                let name = fn_val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let arguments = match fn_val.get("arguments") {
                    Some(Value::String(s)) => s.clone(),
                    Some(val) => val.to_string(),
                    None => "{}".to_string(),
                };

                calls.push(ToolCall {
                    id,
                    call_type,
                    function: FunctionCall { name, arguments },
                });
            }
            if calls.is_empty() { None } else { Some(calls) }
        } else {
            None
        };

        // Extract usage metrics
        let prompt_tokens = resp_json.get("usage")
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let completion_tokens = resp_json.get("usage")
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Ok((content, tool_calls, prompt_tokens, completion_tokens))
    }
}
