//! systemd 服务管理模块

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::process::Command;

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Loading,
    Unknown,
}

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// 服务名称
    pub name: String,
    /// 服务描述
    pub description: String,
    /// 服务状态
    pub status: ServiceStatus,
    /// 是否启用开机启动
    pub enabled: bool,
    /// 主进程 PID
    pub main_pid: Option<u32>,
    /// 激活状态
    pub active_state: String,
    /// 子状态
    pub sub_state: String,
}

/// 服务管理器
pub struct ServiceManager {
    systemctl_path: String,
}

impl ServiceManager {
    /// 创建新的服务管理器
    pub fn new() -> Self {
        Self {
            systemctl_path: "systemctl".to_string(),
        }
    }

    /// 获取所有服务列表
    pub fn get_services(&self) -> Result<Vec<ServiceInfo>> {
        let output = Command::new(&self.systemctl_path)
            .args(["list-units", "--type=service", "--all", "--no-pager", "--no-legend"])
            .output()
            .context("Failed to execute systemctl")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                if name.ends_with(".service") {
                    let load_state = parts.get(1).unwrap_or(&"");
                    let active_state = parts.get(2).unwrap_or(&"");
                    let sub_state = parts.get(3).unwrap_or(&"");
                    let description = parts[4..].join(" ");

                    services.push(ServiceInfo {
                        name: name.clone(),
                        description,
                        status: self.parse_status(*active_state, *sub_state),
                        enabled: self.is_enabled(&name)?,
                        main_pid: None,
                        active_state: active_state.to_string(),
                        sub_state: sub_state.to_string(),
                    });
                }
            }
        }

        Ok(services)
    }

    /// 获取指定服务信息
    pub fn get_service(&self, name: &str) -> Result<ServiceInfo> {
        let output = Command::new(&self.systemctl_path)
            .args(["show", name, "--no-pager"])
            .output()
            .context("Failed to execute systemctl show")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut info = ServiceInfo {
            name: name.to_string(),
            description: String::new(),
            status: ServiceStatus::Unknown,
            enabled: false,
            main_pid: None,
            active_state: String::new(),
            sub_state: String::new(),
        };

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "Description" => info.description = value.to_string(),
                    "ActiveState" => info.active_state = value.to_string(),
                    "SubState" => info.sub_state = value.to_string(),
                    "MainPID" => {
                        info.main_pid = value.parse().ok().filter(|&p| p > 0);
                    }
                    "UnitFileState" => {
                        info.enabled = value == "enabled";
                    }
                    _ => {}
                }
            }
        }

        info.status = self.parse_status(&info.active_state, &info.sub_state);

        Ok(info)
    }

    /// 启动服务
    pub fn start(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["start", name])
            .output()
            .context("Failed to start service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to start service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 停止服务
    pub fn stop(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["stop", name])
            .output()
            .context("Failed to stop service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to stop service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 重启服务
    pub fn restart(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["restart", name])
            .output()
            .context("Failed to restart service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to restart service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 重新加载服务配置
    pub fn reload(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["reload", name])
            .output()
            .context("Failed to reload service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to reload service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 启用开机启动
    pub fn enable(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["enable", name])
            .output()
            .context("Failed to enable service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to enable service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 禁用开机启动
    pub fn disable(&self, name: &str) -> Result<()> {
        let output = Command::new(&self.systemctl_path)
            .args(["disable", name])
            .output()
            .context("Failed to disable service")?;

        if !output.status.success() {
            anyhow::bail!("Failed to disable service: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 检查服务是否启用开机启动
    fn is_enabled(&self, name: &str) -> Result<bool> {
        let output = Command::new(&self.systemctl_path)
            .args(["is-enabled", name])
            .output();

        Ok(output.map(|o| o.status.success()).unwrap_or(false))
    }

    /// 解析服务状态
    fn parse_status(&self, active_state: &str, sub_state: &str) -> ServiceStatus {
        match active_state {
            "active" => match sub_state {
                "running" => ServiceStatus::Running,
                "exited" => ServiceStatus::Stopped,
                _ => ServiceStatus::Running,
            },
            "inactive" | "deactivating" => ServiceStatus::Stopped,
            "failed" => ServiceStatus::Failed,
            "activating" => ServiceStatus::Loading,
            _ => ServiceStatus::Unknown,
        }
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
