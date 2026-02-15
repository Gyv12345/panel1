//! Panel AI - AI Agent 模块
//!
//! 提供安装助手和运维顾问功能

pub mod llm;
pub mod agents;
pub mod tools;

pub use llm::{LlmProvider, LlmMessage, LlmResponse, LlmConfig};
pub use llm::openai::OpenAiProvider;
pub use llm::ollama::OllamaProvider;
pub use agents::{InstallerAgent, AdvisorAgent, AgentResponse};
pub use tools::{ShellTool, DiagnosticTool, ToolResult};

pub mod prelude {
    pub use crate::llm::{LlmProvider, LlmMessage, LlmResponse, LlmConfig};
    pub use crate::llm::openai::OpenAiProvider;
    pub use crate::llm::ollama::OllamaProvider;
    pub use crate::agents::{InstallerAgent, AdvisorAgent, AgentResponse};
    pub use crate::tools::{ShellTool, DiagnosticTool, ToolResult, ToolContext};
}
