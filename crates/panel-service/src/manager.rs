//! 统一服务管理器 - 路由到不同的服务后端

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::binary::{BinaryBackend, UrlInstallMode};
use crate::systemd::SystemdBackend;

/// 服务运行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceMode {
    /// 系统 systemd 服务
    Systemd,
    /// Panel1 托管的二进制
    Panel1,
    /// Docker 容器
    Docker,
}

/// 托管服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedService {
    /// 服务 ID
    pub id: Option<i64>,
    /// 服务名称
    pub name: String,
    /// 服务类型（redis, elasticsearch, etc.）
    pub service_type: String,
    /// 运行模式
    pub mode: ServiceMode,
    /// 版本
    pub version: String,
    /// 二进制路径
    pub binary_path: Option<String>,
    /// 配置文件路径
    pub config_path: Option<String>,
    /// 端口号
    pub port: Option<u16>,
    /// 服务状态
    pub status: ServiceStatus,
    /// 是否自动启动
    pub auto_start: bool,
}

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Starting,
    Stopping,
    Unknown,
}

/// 服务管理器 - 统一管理不同后端的服务
pub struct ServiceManager {
    systemd_backend: SystemdBackend,
    binary_backend: Arc<RwLock<BinaryBackend>>,
}

impl ServiceManager {
    /// 创建新的服务管理器
    pub fn new() -> Self {
        Self {
            systemd_backend: SystemdBackend::new(),
            binary_backend: Arc::new(RwLock::new(BinaryBackend::new())),
        }
    }

    /// 获取所有托管服务列表
    pub async fn list_services(&self) -> Result<Vec<ManagedService>> {
        // TODO: 从数据库获取托管服务列表
        Ok(vec![])
    }

    /// 安装新服务
    pub async fn install_service(
        &self,
        name: &str,
        service_type: &str,
        mode: ServiceMode,
        version: &str,
    ) -> Result<ManagedService> {
        match mode {
            ServiceMode::Systemd => {
                // 检查系统是否已有该服务
                let info = self
                    .systemd_backend
                    .get_service(&format!("{}.service", name))?;
                Ok(ManagedService {
                    id: None,
                    name: name.to_string(),
                    service_type: service_type.to_string(),
                    mode,
                    version: version.to_string(),
                    binary_path: None,
                    config_path: None,
                    port: None,
                    status: if info.status == panel_core::ServiceStatus::Running {
                        ServiceStatus::Running
                    } else {
                        ServiceStatus::Stopped
                    },
                    auto_start: info.enabled,
                })
            }
            ServiceMode::Panel1 => {
                let mut backend = self.binary_backend.write().await;
                let service = backend.install(name, service_type, version).await?;
                Ok(service)
            }
            ServiceMode::Docker => {
                // TODO: 集成 Docker 后端
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
    }

    /// 通过 URL 安装服务（Agent 模式）
    pub async fn install_service_from_url(
        &self,
        url: &str,
        preferred_name: Option<&str>,
        install_mode: UrlInstallMode,
    ) -> Result<ManagedService> {
        let mut backend = self.binary_backend.write().await;
        backend
            .install_from_url(preferred_name, url, install_mode)
            .await
    }

    /// 启动服务
    pub async fn start_service(&self, service: &ManagedService) -> Result<()> {
        match service.mode {
            ServiceMode::Systemd => {
                self.systemd_backend
                    .start(&format!("{}.service", service.name))?;
            }
            ServiceMode::Panel1 => {
                let backend = self.binary_backend.read().await;
                backend.start(service).await?;
            }
            ServiceMode::Docker => {
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
        Ok(())
    }

    /// 停止服务
    pub async fn stop_service(&self, service: &ManagedService) -> Result<()> {
        match service.mode {
            ServiceMode::Systemd => {
                self.systemd_backend
                    .stop(&format!("{}.service", service.name))?;
            }
            ServiceMode::Panel1 => {
                let backend = self.binary_backend.read().await;
                backend.stop(service).await?;
            }
            ServiceMode::Docker => {
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
        Ok(())
    }

    /// 重启服务
    pub async fn restart_service(&self, service: &ManagedService) -> Result<()> {
        match service.mode {
            ServiceMode::Systemd => {
                self.systemd_backend
                    .restart(&format!("{}.service", service.name))?;
            }
            ServiceMode::Panel1 => {
                let backend = self.binary_backend.read().await;
                backend.stop(service).await?;
                backend.start(service).await?;
            }
            ServiceMode::Docker => {
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self, service: &ManagedService) -> Result<ServiceStatus> {
        match service.mode {
            ServiceMode::Systemd => {
                let info = self
                    .systemd_backend
                    .get_service(&format!("{}.service", service.name))?;
                Ok(match info.status {
                    panel_core::ServiceStatus::Running => ServiceStatus::Running,
                    panel_core::ServiceStatus::Stopped => ServiceStatus::Stopped,
                    panel_core::ServiceStatus::Failed => ServiceStatus::Failed,
                    panel_core::ServiceStatus::Loading => ServiceStatus::Starting,
                    panel_core::ServiceStatus::Unknown => ServiceStatus::Unknown,
                })
            }
            ServiceMode::Panel1 => {
                let backend = self.binary_backend.read().await;
                backend.get_status(service).await
            }
            ServiceMode::Docker => {
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
    }

    /// 卸载服务
    pub async fn uninstall_service(&self, service: &ManagedService) -> Result<()> {
        match service.mode {
            ServiceMode::Systemd => {
                self.systemd_backend
                    .disable(&format!("{}.service", service.name))?;
                self.systemd_backend
                    .stop(&format!("{}.service", service.name))?;
            }
            ServiceMode::Panel1 => {
                let backend = self.binary_backend.read().await;
                backend.uninstall(service).await?;
            }
            ServiceMode::Docker => {
                anyhow::bail!("Docker mode not yet implemented");
            }
        }
        Ok(())
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
