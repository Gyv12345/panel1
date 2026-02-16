//! Docker 容器管理

use bollard::container::{
    ListContainersOptions, LogsOptions, RemoveContainerOptions, RestartContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::ContainerSummary;
use bollard::Docker;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

/// 容器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// 容器 ID
    pub id: String,
    /// 容器名称
    pub name: String,
    /// 镜像名称
    pub image: String,
    /// 容器状态
    pub status: ContainerStatus,
    /// 状态文本
    pub status_text: String,
    /// 创建时间
    pub created: i64,
    /// 端口映射
    pub ports: Vec<PortMapping>,
    /// 网络
    pub networks: Vec<String>,
}

/// 容器状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Running,
    Paused,
    Stopped,
    Created,
    Unknown,
}

/// 端口映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String,
    pub host_ip: String,
}

/// Docker 容器管理器
pub struct ContainerManager {
    docker: Docker,
}

impl ContainerManager {
    /// 创建新的容器管理器
    pub async fn new() -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self { docker })
    }

    /// 获取所有容器列表
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>, anyhow::Error> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;

        Ok(containers
            .into_iter()
            .filter_map(|c| self.container_summary_to_info(c))
            .collect())
    }

    /// 启动容器
    pub async fn start_container(&self, id: &str) -> Result<(), anyhow::Error> {
        let options = StartContainerOptions::<String> {
            ..Default::default()
        };
        self.docker.start_container(id, Some(options)).await?;
        Ok(())
    }

    /// 停止容器
    pub async fn stop_container(
        &self,
        id: &str,
        timeout: Option<i32>,
    ) -> Result<(), anyhow::Error> {
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10) as i64,
        };
        self.docker.stop_container(id, Some(options)).await?;
        Ok(())
    }

    /// 重启容器
    pub async fn restart_container(
        &self,
        id: &str,
        timeout: Option<i32>,
    ) -> Result<(), anyhow::Error> {
        let options = RestartContainerOptions {
            t: timeout.unwrap_or(10) as isize,
        };
        self.docker.restart_container(id, Some(options)).await?;
        Ok(())
    }

    /// 删除容器
    pub async fn remove_container(
        &self,
        id: &str,
        force: bool,
        _remove_volumes: bool,
    ) -> Result<(), anyhow::Error> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        self.docker.remove_container(id, Some(options)).await?;
        Ok(())
    }

    /// 获取容器日志
    pub async fn get_container_logs(
        &self,
        id: &str,
        tail: Option<usize>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail
                .map(|t| t.to_string())
                .unwrap_or_else(|| "100".to_string()),
            ..Default::default()
        };

        let mut logs = Vec::new();
        let mut stream = self.docker.logs(id, Some(options));

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let log_line = format!("{}", output);
                    logs.push(log_line);
                }
                Err(e) => {
                    tracing::warn!("Error reading log: {}", e);
                    break;
                }
            }
        }

        Ok(logs)
    }

    /// 转换容器摘要到信息
    fn container_summary_to_info(&self, summary: ContainerSummary) -> Option<ContainerInfo> {
        let id = summary.id?;
        let names = summary.names.unwrap_or_default();
        let name = names
            .first()
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default();

        let status = self.parse_status(summary.state.as_deref().unwrap_or(""));

        let ports = summary
            .ports
            .unwrap_or_default()
            .into_iter()
            .filter_map(|p| {
                Some(PortMapping {
                    container_port: p.private_port,
                    host_port: p.public_port?,
                    protocol: "tcp".to_string(),
                    host_ip: p.ip.unwrap_or_default(),
                })
            })
            .collect();

        Some(ContainerInfo {
            id,
            name,
            image: summary.image.unwrap_or_default(),
            status,
            status_text: summary.status.unwrap_or_default(),
            created: summary.created.unwrap_or(0),
            ports,
            networks: summary
                .network_settings
                .and_then(|n| n.networks.map(|nets| nets.keys().cloned().collect()))
                .unwrap_or_default(),
        })
    }

    /// 解析容器状态
    fn parse_status(&self, state: &str) -> ContainerStatus {
        match state.to_lowercase().as_str() {
            "running" => ContainerStatus::Running,
            "paused" => ContainerStatus::Paused,
            "exited" | "dead" => ContainerStatus::Stopped,
            "created" => ContainerStatus::Created,
            _ => ContainerStatus::Unknown,
        }
    }
}
