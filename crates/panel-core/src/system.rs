//! 系统信息模块 - 获取 CPU、内存、磁盘等系统信息

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

/// 系统基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 主机名
    pub hostname: String,
    /// 操作系统名称
    pub os_name: String,
    /// 操作系统版本
    pub os_version: String,
    /// 内核版本
    pub kernel_version: String,
    /// 系统架构
    pub arch: String,
    /// 系统启动时间 (Unix timestamp)
    pub boot_time: u64,
    /// 系统运行时间 (秒)
    pub uptime: u64,
}

/// CPU 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU 型号
    pub brand: String,
    /// CPU 核心数
    pub cores: usize,
    /// CPU 使用率 (0-100)
    pub usage: f32,
    /// 各核心使用率
    pub core_usages: Vec<f32>,
}

/// 内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总内存 (字节)
    pub total: u64,
    /// 已用内存 (字节)
    pub used: u64,
    /// 可用内存 (字节)
    pub available: u64,
    /// 内存使用率 (0-100)
    pub usage: f32,
    /// 总交换空间 (字节)
    pub swap_total: u64,
    /// 已用交换空间 (字节)
    pub swap_used: u64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// 挂载点
    pub mount_point: String,
    /// 文件系统类型
    pub fs_type: String,
    /// 总空间 (字节)
    pub total: u64,
    /// 已用空间 (字节)
    pub used: u64,
    /// 可用空间 (字节)
    pub available: u64,
    /// 使用率 (0-100)
    pub usage: f32,
}

/// 系统信息获取器
pub struct SystemMonitor {
    sys: System,
    boot_time: u64,
}

impl SystemMonitor {
    /// 创建新的系统监控器
    pub fn new() -> Self {
        let sys = System::new_all();
        // Get boot time from /proc/stat on Linux
        let boot_time = std::fs::read_to_string("/proc/stat")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("btime "))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or(0);

        Self { sys, boot_time }
    }

    /// 刷新系统信息
    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    /// 获取系统基本信息
    pub fn get_system_info(&self) -> SystemInfo {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SystemInfo {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            arch: System::cpu_arch(),
            boot_time: self.boot_time,
            uptime: now.saturating_sub(self.boot_time),
        }
    }

    /// 获取 CPU 信息
    pub fn get_cpu_info(&mut self) -> CpuInfo {
        self.sys.refresh_cpu_all();

        let cpus = self.sys.cpus();
        let brand = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let core_usages: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let total_usage = if !core_usages.is_empty() {
            core_usages.iter().sum::<f32>() / core_usages.len() as f32
        } else {
            0.0
        };

        CpuInfo {
            brand,
            cores: cpus.len(),
            usage: total_usage,
            core_usages,
        }
    }

    /// 获取内存信息
    pub fn get_memory_info(&self) -> MemoryInfo {
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let available = total.saturating_sub(used);
        let usage = if total > 0 {
            (used as f64 / total as f64 * 100.0) as f32
        } else {
            0.0
        };

        MemoryInfo {
            total,
            used,
            available,
            usage,
            swap_total: self.sys.total_swap(),
            swap_used: self.sys.used_swap(),
        }
    }

    /// 获取磁盘信息列表
    pub fn get_disk_info(&self) -> Vec<DiskInfo> {
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();

        disks
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                let usage = if total > 0 {
                    (used as f64 / total as f64 * 100.0) as f32
                } else {
                    0.0
                };

                DiskInfo {
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    fs_type: disk.file_system().to_string_lossy().to_string(),
                    total,
                    used,
                    available,
                    usage,
                }
            })
            .collect()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
