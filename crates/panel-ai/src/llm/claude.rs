//! Claude Provider - 使用 genai 库
//!
//! 支持基于 Panel1 持久化配置或环境变量的多协议接入。

use anyhow::Result;
use async_trait::async_trait;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};

use crate::config::{load_ai_config, AiProtocol};

use super::provider::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};

/// Claude Provider - 使用 genai 库
pub struct ClaudeProvider {
    config: LlmConfig,
    client: Client,
}

/// 解析协议类型。
fn resolve_protocol(
    explicit_protocol: Option<AiProtocol>,
    persisted_protocol: Option<AiProtocol>,
    anthropic_key: Option<&str>,
    openai_key: Option<&str>,
    model_hint: Option<&str>,
) -> AiProtocol {
    explicit_protocol
        .or(persisted_protocol)
        .or_else(|| {
            if anthropic_key.is_some() {
                Some(AiProtocol::Anthropic)
            } else if openai_key.is_some() {
                Some(AiProtocol::Openai)
            } else {
                model_hint.map(infer_protocol_from_model)
            }
        })
        .unwrap_or(AiProtocol::Openai)
}

/// 根据提示模型推断协议。
fn infer_protocol_from_model(model: &str) -> AiProtocol {
    if model.trim().to_ascii_lowercase().starts_with("claude") {
        AiProtocol::Anthropic
    } else {
        AiProtocol::Openai
    }
}

/// 解析模型名称。
fn resolve_model(
    explicit_model: Option<&str>,
    persisted_model: Option<&str>,
    protocol: AiProtocol,
    anthropic_key: Option<&str>,
    openai_key: Option<&str>,
    ollama_model: Option<&str>,
) -> String {
    explicit_model
        .map(ToOwned::to_owned)
        .or_else(|| persisted_model.map(ToOwned::to_owned))
        .or_else(|| {
            if protocol == AiProtocol::Anthropic && anthropic_key.is_some() {
                Some("claude-sonnet-4-5".to_string())
            } else if protocol == AiProtocol::Openai && openai_key.is_some() {
                Some("gpt-4o-mini".to_string())
            } else {
                None
            }
        })
        .or_else(|| ollama_model.map(ToOwned::to_owned))
        .unwrap_or_else(|| protocol.default_model().to_string())
}

/// 解析 API Key。
fn resolve_api_key(
    explicit_api_key: Option<&str>,
    persisted_api_key: Option<&str>,
    gateway_token: Option<&str>,
    anthropic_key: Option<&str>,
    openai_key: Option<&str>,
    protocol: AiProtocol,
) -> Option<String> {
    explicit_api_key
        .map(ToOwned::to_owned)
        .or_else(|| persisted_api_key.map(ToOwned::to_owned))
        .or_else(|| gateway_token.map(ToOwned::to_owned))
        .or_else(|| match protocol {
            AiProtocol::Anthropic => anthropic_key.map(ToOwned::to_owned),
            AiProtocol::Openai => openai_key.map(ToOwned::to_owned),
        })
        .or_else(|| anthropic_key.map(ToOwned::to_owned))
        .or_else(|| openai_key.map(ToOwned::to_owned))
}

/// 协议映射到 genai 适配器类型。
fn adapter_kind_from_protocol(protocol: AiProtocol) -> AdapterKind {
    match protocol {
        AiProtocol::Openai => AdapterKind::OpenAI,
        AiProtocol::Anthropic => AdapterKind::Anthropic,
    }
}

/// 把 API Key 注入到 genai 默认读取的环境变量。
fn set_default_api_key_env(protocol: AiProtocol, api_key: &str) {
    match protocol {
        AiProtocol::Openai => std::env::set_var("OPENAI_API_KEY", api_key),
        AiProtocol::Anthropic => std::env::set_var("ANTHROPIC_API_KEY", api_key),
    }
}

/// 从字符串环境变量解析协议。
fn parse_protocol_env(raw: Option<&str>) -> Option<AiProtocol> {
    raw.and_then(AiProtocol::parse)
}

impl ClaudeProvider {
    /// 创建新的 Claude Provider
    ///
    /// 配置优先级：
    /// 1. `PANEL1_AI_*` 环境变量
    /// 2. `~/.panel1/ai.toml` 持久化配置
    /// 3. 兼容旧变量（`CLAUDE_*` / `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`）
    pub fn new() -> Self {
        let persisted = load_ai_config().ok().flatten();

        // 新变量（Panel1 标准）
        let panel_protocol =
            parse_protocol_env(std::env::var("PANEL1_AI_PROTOCOL").ok().as_deref());
        let panel_base_url = std::env::var("PANEL1_AI_BASE_URL").ok();
        let panel_api_key = std::env::var("PANEL1_AI_API_KEY").ok();
        let panel_model = std::env::var("PANEL1_AI_MODEL").ok();

        // 兼容旧变量
        let legacy_gateway_url = std::env::var("CLAUDE_GATEWAY_URL").ok();
        let legacy_gateway_token = std::env::var("CLAUDE_GATEWAY_TOKEN").ok();
        let legacy_model = std::env::var("CLAUDE_MODEL").ok();
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let openai_key = std::env::var("OPENAI_API_KEY").ok();
        let ollama_model = std::env::var("OLLAMA_MODEL").ok();

        let explicit_model = panel_model.as_deref().or(legacy_model.as_deref());
        let model_hint = explicit_model
            .or_else(|| persisted.as_ref().map(|cfg| cfg.model.as_str()))
            .or(ollama_model.as_deref());

        let protocol = resolve_protocol(
            panel_protocol,
            persisted.as_ref().map(|cfg| cfg.protocol),
            anthropic_key.as_deref(),
            openai_key.as_deref(),
            model_hint,
        );

        let model = resolve_model(
            explicit_model,
            persisted.as_ref().map(|cfg| cfg.model.as_str()),
            protocol,
            anthropic_key.as_deref(),
            openai_key.as_deref(),
            ollama_model.as_deref(),
        );

        let base_url = panel_base_url
            .or_else(|| persisted.as_ref().and_then(|cfg| cfg.base_url.clone()))
            .or(legacy_gateway_url);

        let effective_token = resolve_api_key(
            panel_api_key.as_deref(),
            persisted.as_ref().and_then(|cfg| cfg.api_key.as_deref()),
            legacy_gateway_token.as_deref(),
            anthropic_key.as_deref(),
            openai_key.as_deref(),
            protocol,
        );

        let config = LlmConfig {
            api_key: effective_token.clone(),
            base_url: base_url.clone(),
            model: model.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        let client = if let (Some(url), Some(token)) = (base_url, effective_token.clone()) {
            // 自定义网关模式：URL + key + protocol
            let token_for_closure = token.clone();
            let adapter_kind = adapter_kind_from_protocol(protocol);
            let target_resolver = ServiceTargetResolver::from_resolver_fn(
                move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                    let ServiceTarget { model, .. } = service_target;
                    let endpoint = Endpoint::from_owned(url.clone());
                    let auth = AuthData::from_single(token_for_closure.clone());
                    let model = ModelIden::new(adapter_kind, model.model_name);
                    Ok(ServiceTarget { endpoint, auth, model })
                },
            );
            Client::builder()
                .with_service_target_resolver(target_resolver)
                .build()
        } else {
            if let Some(token) = effective_token {
                // 没有自定义 URL 时，走默认 endpoint；token 通过标准环境变量注入
                set_default_api_key_env(protocol, &token);
            }
            Client::default()
        };

        Self { config, client }
    }

    /// 使用指定模型创建 Provider
    pub fn with_model(model: &str) -> Self {
        std::env::set_var("PANEL1_AI_MODEL", model);
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
        std::env::set_var("PANEL1_AI_PROTOCOL", "openai");
        std::env::set_var("PANEL1_AI_BASE_URL", base_url.into());
        std::env::set_var("PANEL1_AI_API_KEY", auth_token.into());
        std::env::set_var("PANEL1_AI_MODEL", model.into());
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
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    /// 发送多轮消息并获取响应。
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

        let content = chat_res
            .content_text_as_str()
            .unwrap_or_default()
            .to_string();
        let tokens_used = chat_res.usage.completion_tokens.map(|t| t as u32);

        Ok(LlmResponse {
            content,
            tokens_used,
            model: self.config.model.clone(),
        })
    }

    /// 发送单条消息并获取响应。
    async fn send(&self, message: &str) -> Result<LlmResponse> {
        self.chat(vec![LlmMessage::user(message)]).await
    }

    /// 获取当前 Provider 配置。
    fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// 检查当前 Provider 是否可用。
    async fn is_available(&self) -> bool {
        self.config.api_key.is_some()
            || matches!(
                AdapterKind::from_model(&self.config.model),
                Ok(AdapterKind::Ollama)
            )
    }
}

/// 重新导出，保持兼容性
pub use genai::Client as GenaiClient;

#[cfg(test)]
mod tests {
    use super::{infer_protocol_from_model, resolve_api_key, resolve_model, resolve_protocol};
    use crate::config::AiProtocol;

    /// 测试：显式协议应优先生效。
    #[test]
    fn resolve_protocol_prefers_explicit() {
        let protocol = resolve_protocol(
            Some(AiProtocol::Anthropic),
            Some(AiProtocol::Openai),
            Some("anthropic"),
            Some("openai"),
            Some("gpt-4o-mini"),
        );
        assert_eq!(protocol, AiProtocol::Anthropic);
    }

    /// 测试：模型名应优先使用显式配置。
    #[test]
    fn resolve_model_prefers_explicit_model() {
        let model = resolve_model(
            Some("custom-model"),
            Some("persisted-model"),
            AiProtocol::Openai,
            Some("anthropic-key"),
            Some("openai-key"),
            Some("llama3.2"),
        );
        assert_eq!(model, "custom-model");
    }

    /// 测试：无显式模型时按协议选择默认模型。
    #[test]
    fn resolve_model_uses_protocol_default() {
        let model = resolve_model(None, None, AiProtocol::Anthropic, None, None, None);
        assert_eq!(model, "claude-sonnet-4-5");

        let model = resolve_model(None, None, AiProtocol::Openai, None, None, None);
        assert_eq!(model, "gpt-4o-mini");
    }

    /// 测试：解析 API Key 的优先级。
    #[test]
    fn resolve_api_key_prefers_panel_then_persisted_then_legacy() {
        let key = resolve_api_key(
            Some("panel"),
            Some("persisted"),
            Some("gateway"),
            Some("anthropic"),
            Some("openai"),
            AiProtocol::Openai,
        );
        assert_eq!(key, Some("panel".to_string()));

        let key = resolve_api_key(
            None,
            Some("persisted"),
            Some("gateway"),
            Some("anthropic"),
            Some("openai"),
            AiProtocol::Openai,
        );
        assert_eq!(key, Some("persisted".to_string()));

        let key = resolve_api_key(
            None,
            None,
            Some("gateway"),
            Some("anthropic"),
            Some("openai"),
            AiProtocol::Openai,
        );
        assert_eq!(key, Some("gateway".to_string()));
    }

    /// 测试：协议推断逻辑。
    #[test]
    fn infer_protocol_from_model_name() {
        assert_eq!(
            infer_protocol_from_model("claude-3-5-sonnet"),
            AiProtocol::Anthropic
        );
        assert_eq!(
            infer_protocol_from_model("deepseek-chat"),
            AiProtocol::Openai
        );
    }
}
