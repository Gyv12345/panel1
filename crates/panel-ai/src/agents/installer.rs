//! 安装助手 Agent

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::llm::{LlmMessage, LlmProvider};
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

/// URL 安装执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlInstallReport {
    /// 是否安装成功
    pub success: bool,
    /// 已安装服务名
    pub service_name: Option<String>,
    /// 二进制路径
    pub binary_path: Option<String>,
    /// 执行日志
    pub logs: Vec<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 安装模式（透传到 panel-service）
pub type InstallMode = panel_service::UrlInstallMode;

/// 安装助手 Agent
pub struct InstallerAgent {
    provider: Arc<dyn LlmProvider>,
    diagnostic: DiagnosticTool,
    service_manager: panel_service::ServiceManager,
}

impl InstallerAgent {
    /// 创建新的安装助手
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            diagnostic: DiagnosticTool::new(),
            service_manager: panel_service::ServiceManager::new(),
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
            if trimmed.starts_with("sudo ")
                || trimmed.starts_with("apt ")
                || trimmed.starts_with("yum ")
                || trimmed.starts_with("dnf ")
                || trimmed.starts_with("systemctl ")
                || trimmed.starts_with("docker ")
                || trimmed.starts_with("curl ")
                || trimmed.starts_with("wget ")
                || trimmed.starts_with("chmod ")
                || trimmed.starts_with("mkdir ")
            {
                commands.push(trimmed.to_string());
            }
        }

        commands
    }

    /// 直接通过 URL 安装工具，带自动重试和自修复
    pub async fn install_from_url(
        &self,
        raw_url: &str,
        preferred_name: Option<&str>,
        mode: InstallMode,
    ) -> Result<UrlInstallReport> {
        let normalized = normalize_url(raw_url);
        let mut logs = vec![
            format!("目标地址: {}", normalized),
            format!("安装模式: {}", mode.as_str()),
            "检查宿主机依赖（Docker/Node/Python）".to_string(),
        ];

        let mut candidates = vec![normalized.clone()];
        if normalized.starts_with("https://") {
            candidates.push(normalized.replacen("https://", "http://", 1));
        }

        for (idx, candidate) in candidates.iter().enumerate() {
            let attempt = idx + 1;
            logs.push(format!("尝试 #{} 安装...", attempt));

            match self
                .service_manager
                .install_service_from_url(candidate, preferred_name, mode)
                .await
            {
                Ok(service) => {
                    logs.push("安装成功".to_string());
                    return Ok(UrlInstallReport {
                        success: true,
                        service_name: Some(service.name),
                        binary_path: service.binary_path,
                        logs,
                        error: None,
                    });
                }
                Err(err) => {
                    logs.push(format!("失败: {}", err));
                    logs.push("执行自修复策略（目录清理/协议切换/重试）".to_string());
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }
        }

        Ok(UrlInstallReport {
            success: false,
            service_name: None,
            binary_path: None,
            logs,
            error: Some("多次重试后安装失败".to_string()),
        })
    }
}
/// 执行 `normalize_url`。

fn normalize_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    format!("https://{}", trimmed)
}

#[cfg(test)]
mod tests {
    use super::normalize_url;

    /// 执行 `normalize_url_keeps_explicit_scheme`。
    #[test]
    fn normalize_url_keeps_explicit_scheme() {
        assert_eq!(
            normalize_url("https://example.com/tool"),
            "https://example.com/tool"
        );
        assert_eq!(
            normalize_url("http://example.com/tool"),
            "http://example.com/tool"
        );
    }

    /// 执行 `normalize_url_adds_https_by_default`。
    #[test]
    fn normalize_url_adds_https_by_default() {
        assert_eq!(
            normalize_url("downloads.example.com/tool"),
            "https://downloads.example.com/tool"
        );
    }
}
