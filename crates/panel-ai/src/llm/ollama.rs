//! Ollama Provider 实现（本地模型支持）

use async_trait::async_trait;
use anyhow::{Result, Context, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmMessage, LlmResponse, LlmConfig, LlmRole};

/// Ollama API 响应结构
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    model: String,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API 请求结构
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaApiMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaApiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Ollama Provider
pub struct OllamaProvider {
    client: Client,
    config: LlmConfig,
    base_url: String,
}

impl OllamaProvider {
    /// 创建新的 Ollama Provider
    pub fn new(config: LlmConfig) -> Self {
        let base_url = config.base_url.clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();

        Self { client, config, base_url }
    }

    /// 使用默认配置创建
    pub fn with_model(model: &str) -> Self {
        let config = LlmConfig {
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            model: model.to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.7),
        };
        Self::new(config)
    }

    fn get_api_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
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
impl LlmProvider for OllamaProvider {
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse> {
        let api_messages: Vec<OllamaApiMessage> = messages
            .into_iter()
            .map(|m| OllamaApiMessage {
                role: Self::role_to_string(&m.role).to_string(),
                content: m.content,
            })
            .collect();

        let options = OllamaOptions {
            num_predict: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let request = OllamaRequest {
            model: self.config.model.clone(),
            messages: api_messages,
            stream: false,
            options: Some(options),
        };

        let response = self.client
            .post(self.get_api_url())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Ollama API error: {}", error_text);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(LlmResponse {
            content: ollama_response.message.content,
            tokens_used: ollama_response.eval_count,
            model: ollama_response.model,
        })
    }

    fn config(&self) -> &LlmConfig {
        &self.config
    }

    async fn is_available(&self) -> bool {
        // 检查 Ollama 服务是否运行
        match self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
