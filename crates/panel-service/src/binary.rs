//! 二进制后端 - 下载和管理 Panel1 托管的二进制服务

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Child};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::manager::{ManagedService, ServiceStatus};

/// 二进制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    /// 下载 URL 模板
    pub download_url: String,
    /// 校验和（可选）
    pub checksum: Option<String>,
    /// 默认端口
    pub default_port: u16,
    /// 启动命令
    pub start_command: String,
    /// 配置文件模板
    pub config_template: Option<String>,
}

/// 进程守护器
pub struct ProcessGuard {
    /// 进程映射表
    processes: Arc<RwLock<HashMap<String, u32>>>,
}

impl ProcessGuard {
    /// 创建新的进程守护器
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册进程
    pub async fn register(&self, name: &str, pid: u32) {
        let mut processes = self.processes.write().await;
        processes.insert(name.to_string(), pid);
        info!("Registered process {} with PID {}", name, pid);
    }

    /// 注销进程
    pub async fn unregister(&self, name: &str) {
        let mut processes = self.processes.write().await;
        processes.remove(name);
        info!("Unregistered process {}", name);
    }

    /// 获取进程 PID
    pub async fn get_pid(&self, name: &str) -> Option<u32> {
        let processes = self.processes.read().await;
        processes.get(name).copied()
    }

    /// 检查进程是否运行
    pub async fn is_running(&self, name: &str) -> bool {
        if let Some(pid) = self.get_pid(name).await {
            // 检查进程是否存在
            let path = PathBuf::from("/proc").join(pid.to_string());
            path.exists()
        } else {
            false
        }
    }
}

impl Default for ProcessGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// 二进制后端
pub struct BinaryBackend {
    /// 数据目录
    data_dir: PathBuf,
    /// 进程守护器
    process_guard: ProcessGuard,
}

impl BinaryBackend {
    /// 创建新的二进制后端
    pub fn new() -> Self {
        let data_dir = PathBuf::from("/opt/panel/services");
        Self {
            data_dir,
            process_guard: ProcessGuard::new(),
        }
    }

    /// 使用自定义数据目录创建
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            process_guard: ProcessGuard::new(),
        }
    }

    /// 安装服务
    pub async fn install(&mut self, name: &str, service_type: &str, version: &str) -> Result<ManagedService> {
        let service_dir = self.data_dir.join(name);
        let binary_path = service_dir.join(format!("{}-{}", service_type, version));

        // 创建服务目录
        tokio::fs::create_dir_all(&service_dir)
            .await
            .context("Failed to create service directory")?;

        // 检查二进制是否已存在
        if !binary_path.exists() {
            // TODO: 下载二进制
            info!("Would download {} version {} to {:?}", service_type, version, binary_path);
        }

        Ok(ManagedService {
            id: None,
            name: name.to_string(),
            service_type: service_type.to_string(),
            mode: crate::manager::ServiceMode::Panel1,
            version: version.to_string(),
            binary_path: Some(binary_path.to_string_lossy().to_string()),
            config_path: None,
            port: None,
            status: ServiceStatus::Stopped,
            auto_start: false,
        })
    }

    /// 启动服务
    pub async fn start(&self, service: &ManagedService) -> Result<()> {
        let binary_path = service.binary_path.as_ref()
            .context("Binary path not set")?;

        if !Path::new(binary_path).exists() {
            bail!("Binary not found: {}", binary_path);
        }

        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(binary_path, std::fs::Permissions::from_mode(0o755))
                .await
                .context("Failed to set executable permissions")?;
        }

        // 启动进程
        let child = Command::new(binary_path)
            .current_dir(self.data_dir.join(&service.name))
            .spawn()
            .context("Failed to start service")?;

        let pid = child.id();
        self.process_guard.register(&service.name, pid).await;

        info!("Started service {} with PID {}", service.name, pid);
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&self, service: &ManagedService) -> Result<()> {
        if let Some(pid) = self.process_guard.get_pid(&service.name).await {
            // 发送 SIGTERM
            #[cfg(unix)]
            {
                use std::process::Command as StdCommand;
                let _ = StdCommand::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
            }

            // 等待进程退出
            for _ in 0..10 {
                if !self.process_guard.is_running(&service.name).await {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // 如果还在运行，强制杀死
            if self.process_guard.is_running(&service.name).await {
                #[cfg(unix)]
                {
                    use std::process::Command as StdCommand;
                    let _ = StdCommand::new("kill")
                        .arg("-KILL")
                        .arg(pid.to_string())
                        .output();
                }
            }

            self.process_guard.unregister(&service.name).await;
            info!("Stopped service {}", service.name);
        }

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self, service: &ManagedService) -> Result<ServiceStatus> {
        if self.process_guard.is_running(&service.name).await {
            Ok(ServiceStatus::Running)
        } else {
            Ok(ServiceStatus::Stopped)
        }
    }

    /// 卸载服务
    pub async fn uninstall(&self, service: &ManagedService) -> Result<()> {
        // 先停止服务
        self.stop(service).await?;

        // 删除服务目录
        let service_dir = self.data_dir.join(&service.name);
        if service_dir.exists() {
            tokio::fs::remove_dir_all(&service_dir)
                .await
                .context("Failed to remove service directory")?;
        }

        info!("Uninstalled service {}", service.name);
        Ok(())
    }
}

impl Default for BinaryBackend {
    fn default() -> Self {
        Self::new()
    }
}
