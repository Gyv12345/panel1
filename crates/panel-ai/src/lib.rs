//! Panel AI - AI Agent 模块
//!
//! 提供安装助手和运维顾问功能

pub mod agents;
pub mod llm;
pub mod tools;

pub use agents::{AdvisorAgent, AgentResponse, InstallerAgent};
pub use llm::ollama::OllamaProvider;
pub use llm::openai::OpenAiProvider;
pub use llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
pub use tools::{DiagnosticTool, ShellTool, ToolResult};

pub mod prelude {
    pub use crate::agents::{AdvisorAgent, AgentResponse, InstallerAgent};
    pub use crate::llm::ollama::OllamaProvider;
    pub use crate::llm::openai::OpenAiProvider;
    pub use crate::llm::{LlmConfig, LlmMessage, LlmProvider, LlmResponse};
    pub use crate::tools::{DiagnosticTool, ShellTool, ToolContext, ToolResult};
}
