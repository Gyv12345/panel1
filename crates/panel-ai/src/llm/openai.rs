//! OpenAI Provider 实现

use async_trait::async_trait;
use anyhow::{Result, Context, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmMessage, LlmResponse, LlmConfig, LlmRole};

/// OpenAI API 响应结构
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    total_tokens: u32,
}

/// OpenAI API 请求结构
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAiApiMessage {
    role: String,
    content: String,
}

/// OpenAI Provider
pub struct OpenAiProvider {
    client: Client,
    config: LlmConfig,
}

impl OpenAiProvider {
    /// 创建新的 OpenAI Provider
    pub fn new(config: LlmConfig) -> Result<Self> {
        if config.api_key.is_none() {
            bail!("OpenAI API key is required");
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// 使用环境变量创建
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        let base_url = std::env::var("OPENAI_BASE_URL").ok();

        let config = LlmConfig {
            api_key,
            base_url,
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            max_tokens: Some(2048),
            temperature: Some(0.7),
        };

        Self::new(config)
    }

    fn get_api_url(&self) -> String {
        self.config.base_url.as_ref()
            .map(|u| format!("{}/chat/completions", u.trim_end_matches('/')))
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string())
    }

    fn role_to_string(role: &LlmRole) -> &'static str {
        match role {
            LlmRole::System => "system",
            LlmRole::User => "user",
            LlmRole::Assistant => "assistant",
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .context("OpenAI API key not set")?;

        let api_messages: Vec<OpenAiApiMessage> = messages
            .into_iter()
            .map(|m| OpenAiApiMessage {
                role: Self::role_to_string(&m.role).to_string(),
                content: m.content,
            })
            .collect();

        let request = OpenAiRequest {
            model: self.config.model.clone(),
            messages: api_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let response = self.client
            .post(self.get_api_url())
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            bail!("OpenAI API error: {}", error_text);
        }

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let content = openai_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            tokens_used: openai_response.usage.map(|u| u.total_tokens),
            model: openai_response.model,
        })
    }

    fn config(&self) -> &LlmConfig {
        &self.config
    }

    async fn is_available(&self) -> bool {
        if self.config.api_key.is_none() {
            return false;
        }

        // 尝试一个简单的请求
        (self.send("Hi").await).is_ok()
    }
}
