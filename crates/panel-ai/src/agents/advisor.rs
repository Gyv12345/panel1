//! 运维顾问 Agent

use anyhow::Result;
use std::sync::Arc;

use crate::agents::installer::AgentResponse;
use crate::llm::{LlmMessage, LlmProvider};
use crate::tools::{DiagnosticTool, ToolContext};

/// 运维顾问 Agent
pub struct AdvisorAgent {
    provider: Arc<dyn LlmProvider>,
    diagnostic: DiagnosticTool,
}

impl AdvisorAgent {
    /// 创建新的运维顾问
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            diagnostic: DiagnosticTool::new(),
        }
    }

    /// 系统诊断
    pub async fn diagnose_system(&self) -> Result<AgentResponse> {
        let ctx = ToolContext::default();
        let diagnostics = self.diagnostic.run_full_diagnosis(&ctx).await?;

        let system_prompt = r#"你是一个专业的 Linux 系统运维顾问。你的任务是：
1. 分析系统状态，识别潜在问题
2. 提供优化建议
3. 推荐最佳实践
4. 警告可能的风险

请用中文回复，分析要专业且易于理解。"#;

        let user_message = format!(
            "请分析以下系统诊断信息并给出建议：\n\n{}",
            serde_json::to_string_pretty(&diagnostics)?
        );

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(&user_message),
        ];

        let response = self.provider.chat(messages).await?;

        Ok(AgentResponse {
            content: response.content,
            suggested_commands: vec![],
            requires_confirmation: false,
        })
    }

    /// 性能优化建议
    pub async fn get_performance_advice(&self) -> Result<AgentResponse> {
        let ctx = ToolContext::default();
        let perf_info = self.diagnostic.get_performance_info(&ctx).await?;

        let system_prompt = r#"你是一个 Linux 性能优化专家。基于提供的性能数据：
1. 识别性能瓶颈
2. 提供具体的优化建议
3. 给出配置参数建议
4. 评估优化风险

请用中文回复，建议要具体可操作。"#;

        let user_message = format!(
            "请分析以下性能数据并给出优化建议：\n\n{}",
            serde_json::to_string_pretty(&perf_info)?
        );

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(&user_message),
        ];

        let response = self.provider.chat(messages).await?;

        Ok(AgentResponse {
            content: response.content,
            suggested_commands: vec![],
            requires_confirmation: false,
        })
    }

    /// 安全检查
    pub async fn security_check(&self) -> Result<AgentResponse> {
        let ctx = ToolContext::default();
        let security_info = self.diagnostic.get_security_info(&ctx).await?;

        let system_prompt = r#"你是一个 Linux 安全专家。基于提供的安全信息：
1. 识别安全风险
2. 提供加固建议
3. 推荐安全最佳实践
4. 评估风险等级

请用中文回复，区分高、中、低风险。"#;

        let user_message = format!(
            "请分析以下安全信息并给出加固建议：\n\n{}",
            serde_json::to_string_pretty(&security_info)?
        );

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(&user_message),
        ];

        let response = self.provider.chat(messages).await?;

        Ok(AgentResponse {
            content: response.content,
            suggested_commands: vec![],
            requires_confirmation: false,
        })
    }

    /// 自由问答
    pub async fn ask(&self, question: &str) -> Result<AgentResponse> {
        let system_prompt = r#"你是一个专业的 Linux 服务器运维顾问。
你可以帮助用户：
1. 解决服务器运维问题
2. 提供配置建议
3. 诊断系统故障
4. 推荐最佳实践

请用中文回复，尽量提供具体的命令和配置示例。"#;

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(question),
        ];

        let response = self.provider.chat(messages).await?;

        Ok(AgentResponse {
            content: response.content,
            suggested_commands: vec![],
            requires_confirmation: false,
        })
    }
}
