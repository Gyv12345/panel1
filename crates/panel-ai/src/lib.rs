//! Panel AI - AI Agent 模块
//!
//! 提供安装助手和运维顾问功能

pub mod agents;
pub mod llm;
pub mod tools;

pub use agents::{AdvisorAgent, AgentResponse, InstallerAgent, UrlInstallReport};
pub use llm::{ClaudeProvider, GenaiClient};
pub use llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
pub use tools::{DiagnosticTool, ShellTool, ToolResult};

pub mod prelude {
    pub use crate::agents::{AdvisorAgent, AgentResponse, InstallerAgent, UrlInstallReport};
    pub use crate::llm::{ClaudeProvider, GenaiClient};
    pub use crate::llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
    pub use crate::tools::{DiagnosticTool, ShellTool, ToolContext, ToolResult};
}
