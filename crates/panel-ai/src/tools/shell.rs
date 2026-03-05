//! Shell 命令执行工具（带白名单）

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Shell 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
}

/// 允许执行的命令白名单
const ALLOWED_COMMANDS: &[&str] = &[
    // 系统信息
    "uname",
    "hostname",
    "uptime",
    "date",
    "whoami",
    "id",
    // 文件系统
    "ls",
    "cat",
    "head",
    "tail",
    "find",
    "du",
    "df",
    // 进程管理
    "ps",
    "top",
    "htop",
    "pgrep",
    // 网络工具
    "netstat",
    "ss",
    "ip",
    "ping",
    "curl",
    "wget",
    // 服务状态
    "systemctl",
    "journalctl",
    // Docker
    "docker",
    // 包管理器（只读）
    "apt",
    "yum",
    "dnf",
    "dpkg",
    "rpm",
];

/// 危险命令黑名单
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf",
    "mkfs",
    "dd if=",
    "> /dev/sd",
    ":(){ :|:& };:",
    "chmod -R 777",
    "chown -R",
    "iptables -F",
    "userdel",
    "groupdel",
];

/// Shell 工具
pub struct ShellTool {
    /// 是否启用危险命令确认
    require_confirmation: bool,
}

impl ShellTool {
    /// 创建新的 Shell 工具
    pub fn new() -> Self {
        Self {
            require_confirmation: true,
        }
    }

    /// 执行命令
    pub fn execute(&self, command: &str) -> Result<ShellResult> {
        // 解析命令
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            bail!("Empty command");
        }

        let cmd_name = parts[0];

        // 检查白名单
        if !ALLOWED_COMMANDS.contains(&cmd_name) {
            bail!("Command '{}' is not in the allowed list", cmd_name);
        }

        // 检查危险模式
        for pattern in DANGEROUS_PATTERNS {
            if command.contains(pattern) {
                bail!("Command contains dangerous pattern: {}", pattern);
            }
        }

        // 执行命令
        let output = Command::new(cmd_name).args(&parts[1..]).output();

        match output {
            Ok(output) => Ok(ShellResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            }),
            Err(e) => bail!("Failed to execute command: {}", e),
        }
    }

    /// 执行命令（允许危险命令但需要确认）
    pub fn execute_with_confirmation(&self, command: &str) -> Result<ShellResult> {
        if !self.require_confirmation {
            return self.execute(command);
        }

        // 检查危险模式
        for pattern in DANGEROUS_PATTERNS {
            if command.contains(pattern) {
                bail!(
                    "Command '{}' contains dangerous pattern '{}' and requires explicit user confirmation",
                    command, pattern
                );
            }
        }

        self.execute(command)
    }

    /// 检查命令是否安全
    pub fn is_safe_command(&self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let cmd_name = parts[0];

        if !ALLOWED_COMMANDS.contains(&cmd_name) {
            return false;
        }

        for pattern in DANGEROUS_PATTERNS {
            if command.contains(pattern) {
                return false;
            }
        }

        true
    }

    /// 获取允许的命令列表
    pub fn get_allowed_commands(&self) -> &'static [&'static str] {
        ALLOWED_COMMANDS
    }
}

impl Default for ShellTool {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}
