//! Systemd 后端 - 封装 panel-core 的服务管理

use anyhow::Result;
use panel_core::ServiceManager as CoreServiceManager;

/// Systemd 后端
pub struct SystemdBackend {
    inner: CoreServiceManager,
}

impl SystemdBackend {
    /// 创建新的 Systemd 后端
    pub fn new() -> Self {
        Self {
            inner: CoreServiceManager::new(),
        }
    }

    /// 获取服务信息
    pub fn get_service(&self, name: &str) -> Result<panel_core::ServiceInfo> {
        self.inner.get_service(name)
    }

    /// 获取所有服务列表
    pub fn get_services(&self) -> Result<Vec<panel_core::ServiceInfo>> {
        self.inner.get_services()
    }

    /// 启动服务
    pub fn start(&self, name: &str) -> Result<()> {
        self.inner.start(name)
    }

    /// 停止服务
    pub fn stop(&self, name: &str) -> Result<()> {
        self.inner.stop(name)
    }

    /// 重启服务
    pub fn restart(&self, name: &str) -> Result<()> {
        self.inner.restart(name)
    }

    /// 重新加载服务配置
    pub fn reload(&self, name: &str) -> Result<()> {
        self.inner.reload(name)
    }

    /// 启用开机启动
    pub fn enable(&self, name: &str) -> Result<()> {
        self.inner.enable(name)
    }

    /// 禁用开机启动
    pub fn disable(&self, name: &str) -> Result<()> {
        self.inner.disable(name)
    }
}

impl Default for SystemdBackend {
    fn default() -> Self {
        Self::new()
    }
}
