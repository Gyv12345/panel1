//! LLM Provider trait 和通用数据结构

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// LLM 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmRole {
    System,
    User,
    Assistant,
}

/// LLM 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    /// 消息角色
    pub role: LlmRole,
    /// 消息内容
    pub content: String,
}

impl LlmMessage {
    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: LlmRole::System,
            content: content.into(),
        }
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: LlmRole::User,
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: LlmRole::Assistant,
            content: content.into(),
        }
    }
}

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// 响应内容
    pub content: String,
    /// 使用的 token 数（可选）
    pub tokens_used: Option<u32>,
    /// 模型名称
    pub model: String,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API Key
    pub api_key: Option<String>,
    /// API 基础 URL
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            model: "gpt-4o-mini".to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.7),
        }
    }
}

/// LLM Provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 发送消息并获取响应
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse>;

    /// 发送单条消息
    async fn send(&self, message: &str) -> Result<LlmResponse> {
        self.chat(vec![LlmMessage::user(message)]).await
    }

    /// 获取配置
    fn config(&self) -> &LlmConfig;

    /// 检查是否可用
    async fn is_available(&self) -> bool;
}
