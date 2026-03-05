//! Panel AI - AI Agent 模块
//!
//! 提供安装助手和运维顾问功能

pub mod agents;
pub mod config;
pub mod llm;
pub mod tools;

pub use agents::{AdvisorAgent, AgentResponse, InstallerAgent, UrlInstallReport};
pub use config::{
    builtin_model_presets, config_file_path, load_ai_config, load_ai_store, save_ai_config,
    save_ai_store, seed_builtin_profiles, AiConfig, AiConfigStore, AiModelPreset, AiProfile,
    AiProtocol,
};
pub use llm::{ClaudeProvider, GenaiClient};
pub use llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
pub use tools::{DiagnosticTool, ShellTool, ToolResult};
/// URL 安装模式（auto / panel1 / docker），用于在 AI 层透传安装策略。
pub type InstallMode = panel_service::UrlInstallMode;

pub mod prelude {
    pub use crate::agents::{AdvisorAgent, AgentResponse, InstallerAgent, UrlInstallReport};
    pub use crate::config::{
        builtin_model_presets, config_file_path, load_ai_config, load_ai_store, save_ai_config,
        save_ai_store, seed_builtin_profiles, AiConfig, AiConfigStore, AiModelPreset, AiProfile,
        AiProtocol,
    };
    pub use crate::llm::{ClaudeProvider, GenaiClient};
    pub use crate::llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
    pub use crate::tools::{DiagnosticTool, ShellTool, ToolContext, ToolResult};
    pub use crate::InstallMode;
}
