//! Claude Provider - 使用 genai 库
//!
//! 提供与 Anthropic Claude API 的集成，支持自定义网关配置

use anyhow::Result;
use async_trait::async_trait;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};

use super::provider::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};

/// Claude Provider - 使用 genai 库
pub struct ClaudeProvider {
    config: LlmConfig,
    client: Client,
}

impl ClaudeProvider {
    /// 创建新的 Claude Provider
    ///
    /// 优先级：
    /// 1. CLAUDE_GATEWAY_URL + CLAUDE_GATEWAY_TOKEN (网关模式)
    /// 2. ANTHROPIC_API_KEY (直连模式)
    pub fn new() -> Self {
        let gateway_url = std::env::var("CLAUDE_GATEWAY_URL").ok();
        let gateway_token = std::env::var("CLAUDE_GATEWAY_TOKEN").ok();
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let model =
            std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());

        // 优先使用网关 token，否则使用 API key
        let effective_token = gateway_token.or(api_key);

        let config = LlmConfig {
            api_key: effective_token.clone(),
            base_url: gateway_url.clone(),
            model: model.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        let client = if let (Some(url), Some(token)) = (gateway_url, effective_token) {
            // 网关模式：使用自定义 endpoint
            let token_for_closure = token.clone();
            let target_resolver = ServiceTargetResolver::from_resolver_fn(
                move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                    let ServiceTarget { model, .. } = service_target;
                    let endpoint = Endpoint::from_owned(url.clone());
                    let auth = AuthData::from_single(token_for_closure.clone());
                    // 尝试使用 OpenAI adapter kind（大多数网关兼容 OpenAI API）
                    let model = ModelIden::new(AdapterKind::OpenAI, model.model_name);
                    Ok(ServiceTarget { endpoint, auth, model })
                },
            );
            Client::builder()
                .with_service_target_resolver(target_resolver)
                .build()
        } else {
            // 直连模式：使用默认配置
            Client::default()
        };

        Self { config, client }
    }

    /// 使用指定模型创建 Provider
    pub fn with_model(model: &str) -> Self {
        std::env::set_var("CLAUDE_MODEL", model);
        Self::new()
    }

    /// 使用配置创建 Provider
    pub fn with_config(config: LlmConfig) -> Self {
        let mut provider = Self::new();
        provider.config = config;
        provider
    }

    /// 创建支持自定义网关的 Provider
    pub fn with_gateway(
        base_url: impl Into<String>,
        auth_token: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        std::env::set_var("CLAUDE_GATEWAY_URL", base_url.into());
        std::env::set_var("CLAUDE_GATEWAY_TOKEN", auth_token.into());
        std::env::set_var("CLAUDE_MODEL", model.into());
        Self::new()
    }

    /// 将 LlmMessage 转换为 genai ChatMessage
    fn to_genai_message(msg: &LlmMessage) -> ChatMessage {
        match msg.role {
            super::provider::LlmRole::System => ChatMessage::system(&msg.content),
            super::provider::LlmRole::User => ChatMessage::user(&msg.content),
            super::provider::LlmRole::Assistant => ChatMessage::assistant(&msg.content),
        }
    }
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse> {
        let genai_messages: Vec<ChatMessage> =
            messages.iter().map(Self::to_genai_message).collect();
        let chat_req = ChatRequest::new(genai_messages);

        let chat_options = ChatOptions::default()
            .with_max_tokens(self.config.max_tokens.unwrap_or(4096))
            .with_temperature(self.config.temperature.unwrap_or(0.7) as f64);

        let chat_res = self
            .client
            .exec_chat(&self.config.model, chat_req, Some(&chat_options))
            .await?;

        // 获取响应内容
        let content = chat_res
            .content_text_as_str()
            .unwrap_or_default()
            .to_string();

        // 获取 token 使用情况
        let tokens_used = chat_res.usage.completion_tokens.map(|t| t as u32);

        Ok(LlmResponse {
            content,
            tokens_used,
            model: self.config.model.clone(),
        })
    }

    async fn send(&self, message: &str) -> Result<LlmResponse> {
        self.chat(vec![LlmMessage::user(message)]).await
    }

    fn config(&self) -> &LlmConfig {
        &self.config
    }

    async fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

/// 重新导出，保持兼容性
pub use genai::Client as GenaiClient;
