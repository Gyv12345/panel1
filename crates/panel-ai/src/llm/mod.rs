//! LLM Provider 模块

mod provider;
pub mod openai;
pub mod ollama;

pub use provider::{LlmProvider, LlmMessage, LlmResponse, LlmConfig, LlmRole};
pub use openai::OpenAiProvider;
pub use ollama::OllamaProvider;
