//! 系统监控服务

use panel_core::{SystemMonitor, ProcessManager, NetworkManager, ServiceManager};
use panel_core::{SystemInfo, CpuInfo, MemoryInfo, DiskInfo};
use panel_core::{ProcessInfo, NetworkInterface, ServiceInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 系统监控服务
pub struct SystemService {
    system_monitor: Arc<Mutex<SystemMonitor>>,
    process_manager: Arc<Mutex<ProcessManager>>,
    network_manager: Arc<Mutex<NetworkManager>>,
    service_manager: ServiceManager,
}

impl SystemService {
    pub fn new() -> Self {
        Self {
            system_monitor: Arc::new(Mutex::new(SystemMonitor::new())),
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            network_manager: Arc::new(Mutex::new(NetworkManager::new())),
            service_manager: ServiceManager::new(),
        }
    }

    /// 获取系统基本信息
    pub async fn get_system_info(&self) -> SystemInfo {
        let mut monitor = self.system_monitor.lock().await;
        monitor.refresh();
        monitor.get_system_info()
    }

    /// 获取 CPU 信息
    pub async fn get_cpu_info(&self) -> CpuInfo {
        let mut monitor = self.system_monitor.lock().await;
        monitor.get_cpu_info()
    }

    /// 获取内存信息
    pub async fn get_memory_info(&self) -> MemoryInfo {
        let monitor = self.system_monitor.lock().await;
        monitor.get_memory_info()
    }

    /// 获取磁盘信息
    pub async fn get_disk_info(&self) -> Vec<DiskInfo> {
        let monitor = self.system_monitor.lock().await;
        monitor.get_disk_info()
    }

    /// 获取进程列表
    pub async fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut manager = self.process_manager.lock().await;
        manager.get_processes()
    }

    /// 获取网络接口信息
    pub async fn get_network_interfaces(&self) -> Vec<NetworkInterface> {
        let mut manager = self.network_manager.lock().await;
        manager.get_interfaces()
    }

    /// 获取服务列表
    pub fn get_services(&self) -> Result<Vec<ServiceInfo>, anyhow::Error> {
        self.service_manager.get_services()
    }
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new()
    }
}
