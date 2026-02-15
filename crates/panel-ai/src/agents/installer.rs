//! 安装助手 Agent

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::llm::{LlmProvider, LlmMessage};
use crate::tools::{DiagnosticTool, ToolContext};

/// Agent 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// 响应内容
    pub content: String,
    /// 建议的命令（可选）
    pub suggested_commands: Vec<String>,
    /// 需要确认的操作
    pub requires_confirmation: bool,
}

/// 安装助手 Agent
pub struct InstallerAgent {
    provider: Arc<dyn LlmProvider>,
    diagnostic: DiagnosticTool,
}

impl InstallerAgent {
    /// 创建新的安装助手
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            diagnostic: DiagnosticTool::new(),
        }
    }

    /// 获取安装建议
    pub async fn get_install_advice(&self, service_name: &str) -> Result<AgentResponse> {
        let ctx = ToolContext::default();
        let system_info = self.diagnostic.get_system_info(&ctx).await?;

        let system_prompt = r#"你是一个专业的 Linux 服务器安装助手。你的任务是：
1. 根据用户的系统环境，推荐最佳的安装方式
2. 提供具体的安装命令
3. 说明安装后的配置步骤
4. 提醒可能遇到的问题和解决方案

请用中文回复，并提供清晰、可执行的命令。"#;

        let user_message = format!(
            "我想安装 {}。系统信息如下：\n{}\n\n请给出安装建议。",
            service_name, system_info
        );

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(&user_message),
        ];

        let response = self.provider.chat(messages).await?;

        // 解析响应中的命令
        let suggested_commands = self.extract_commands(&response.content);

        Ok(AgentResponse {
            content: response.content,
            suggested_commands,
            requires_confirmation: true,
        })
    }

    /// 生成安装脚本
    pub async fn generate_install_script(
        &self,
        service_name: &str,
        version: Option<&str>,
        mode: &str, // systemd, panel1, docker
    ) -> Result<String> {
        let system_prompt = r#"你是一个专业的脚本生成器。根据用户的需求生成完整的安装脚本。
脚本应该：
1. 包含必要的错误处理
2. 检查依赖项
3. 创建必要的服务用户
4. 配置自动启动
5. 验证安装成功

只输出脚本内容，不要有其他解释。"#;

        let user_message = format!(
            "生成安装 {} {} 的脚本，使用 {} 模式。",
            service_name,
            version.unwrap_or("最新版本"),
            mode
        );

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(&user_message),
        ];

        let response = self.provider.chat(messages).await?;
        Ok(response.content)
    }

    /// 从响应中提取命令
    fn extract_commands(&self, content: &str) -> Vec<String> {
        let mut commands = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            // 检测代码块中的命令
            if trimmed.starts_with("sudo ") ||
               trimmed.starts_with("apt ") ||
               trimmed.starts_with("yum ") ||
               trimmed.starts_with("dnf ") ||
               trimmed.starts_with("systemctl ") ||
               trimmed.starts_with("docker ") ||
               trimmed.starts_with("curl ") ||
               trimmed.starts_with("wget ") ||
               trimmed.starts_with("chmod ") ||
               trimmed.starts_with("mkdir ") {
                commands.push(trimmed.to_string());
            }
        }

        commands
    }
}
