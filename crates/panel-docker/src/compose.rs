//! Docker Compose 管理

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Compose 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeService {
    /// 服务名称
    pub name: String,
    /// 镜像
    pub image: String,
    /// 状态
    pub status: String,
}

/// Docker Compose 管理器
pub struct ComposeManager;

impl ComposeManager {
    /// 创建新的 Compose 管理器
    pub fn new() -> Self {
        Self
    }

    /// 启动项目
    pub fn up(&self, project_dir: &Path, detached: bool, build: bool) -> Result<()> {
        let mut args = vec!["compose", "up"];
        if detached {
            args.push("-d");
        }
        if build {
            args.push("--build");
        }

        let output = Command::new("docker")
            .args(&args)
            .current_dir(project_dir)
            .output()
            .context("Failed to run docker compose up")?;

        if !output.status.success() {
            anyhow::bail!(
                "docker compose up failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// 停止项目
    pub fn down(&self, project_dir: &Path) -> Result<()> {
        let output = Command::new("docker")
            .args(["compose", "down"])
            .current_dir(project_dir)
            .output()
            .context("Failed to run docker compose down")?;

        if !output.status.success() {
            anyhow::bail!(
                "docker compose down failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// 重启项目
    pub fn restart(&self, project_dir: &Path) -> Result<()> {
        let output = Command::new("docker")
            .args(["compose", "restart"])
            .current_dir(project_dir)
            .output()
            .context("Failed to run docker compose restart")?;

        if !output.status.success() {
            anyhow::bail!(
                "docker compose restart failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}

impl Default for ComposeManager {
    fn default() -> Self {
        Self::new()
    }
}
