//! 系统诊断工具

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use super::shell::ShellTool;

/// 工具执行上下文
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 额外参数
    pub params: HashMap<String, String>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 工具名称
    pub tool: String,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub data: serde_json::Value,
    /// 错误信息
    pub error: Option<String>,
}

/// 完整诊断结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// 系统信息
    pub system: SystemDiagnosis,
    /// CPU 诊断
    pub cpu: CpuDiagnosis,
    /// 内存诊断
    pub memory: MemoryDiagnosis,
    /// 磁盘诊断
    pub disk: Vec<DiskDiagnosis>,
    /// 网络诊断
    pub network: NetworkDiagnosis,
    /// 服务诊断
    pub services: Vec<ServiceDiagnosis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiagnosis {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub load_average: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuDiagnosis {
    pub model: String,
    pub cores: usize,
    pub usage_percent: f32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDiagnosis {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub usage_percent: f32,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDiagnosis {
    pub mount_point: String,
    pub fs_type: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnosis {
    pub interfaces: Vec<String>,
    pub listening_ports: Vec<u16>,
    pub established_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiagnosis {
    pub name: String,
    pub status: String,
    pub enabled: bool,
}

/// 性能信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInfo {
    pub cpu: CpuPerfInfo,
    pub memory: MemoryPerfInfo,
    pub disk_io: DiskIoInfo,
    pub network_io: NetworkIoInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuPerfInfo {
    pub usage_percent: f32,
    pub user_percent: f32,
    pub system_percent: f32,
    pub iowait_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPerfInfo {
    pub usage_percent: f32,
    pub cache_mb: u64,
    pub buffers_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoInfo {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoInfo {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

/// 安全信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub ssh_config: SshConfig,
    pub firewall_status: String,
    pub open_ports: Vec<PortInfo>,
    pub failed_logins: usize,
    pub sudo_users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub permit_root_login: bool,
    pub password_auth: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub service: String,
}

/// 诊断工具
pub struct DiagnosticTool {
    shell: ShellTool,
    monitor: RefCell<panel_core::SystemMonitor>,
}

impl DiagnosticTool {
    /// 创建新的诊断工具
    pub fn new() -> Self {
        Self {
            shell: ShellTool::new(),
            monitor: RefCell::new(panel_core::SystemMonitor::new()),
        }
    }

    /// 获取系统信息字符串
    pub async fn get_system_info(&self, _ctx: &ToolContext) -> Result<String> {
        let monitor = self.monitor.borrow();
        let info = monitor.get_system_info();
        drop(monitor); // 释放 borrow

        let mut monitor = self.monitor.borrow_mut();
        let cpu = monitor.get_cpu_info();
        drop(monitor);

        let monitor = self.monitor.borrow();
        let memory = monitor.get_memory_info();
        let disks = monitor.get_disk_info();

        let mut result = format!(
            "主机名: {}\n操作系统: {} {}\n内核版本: {}\n架构: {}\n运行时间: {} 秒\n\n",
            info.hostname,
            info.os_name,
            info.os_version,
            info.kernel_version,
            info.arch,
            info.uptime
        );

        result.push_str(&format!(
            "CPU: {} ({} 核心)\nCPU 使用率: {:.1}%\n\n",
            cpu.brand, cpu.cores, cpu.usage
        ));

        result.push_str(&format!(
            "内存: {:.1} GB / {:.1} GB ({:.1}%)\n交换空间: {:.1} GB / {:.1} GB\n\n",
            memory.used as f64 / 1024.0 / 1024.0 / 1024.0,
            memory.total as f64 / 1024.0 / 1024.0 / 1024.0,
            memory.usage,
            memory.swap_used as f64 / 1024.0 / 1024.0 / 1024.0,
            memory.swap_total as f64 / 1024.0 / 1024.0 / 1024.0
        ));

        result.push_str("磁盘使用情况:\n");
        for disk in disks {
            result.push_str(&format!(
                "  {} ({}) {:.1} GB / {:.1} GB ({:.1}%)\n",
                disk.mount_point,
                disk.fs_type,
                disk.used as f64 / 1024.0 / 1024.0 / 1024.0,
                disk.total as f64 / 1024.0 / 1024.0 / 1024.0,
                disk.usage
            ));
        }

        Ok(result)
    }

    /// 运行完整诊断
    pub async fn run_full_diagnosis(&self, _ctx: &ToolContext) -> Result<DiagnosisResult> {
        let monitor = self.monitor.borrow();
        let sys_info = monitor.get_system_info();
        let disk_info = monitor.get_disk_info();
        let mem_info = monitor.get_memory_info();
        drop(monitor);

        let mut monitor = self.monitor.borrow_mut();
        let cpu_info = monitor.get_cpu_info();
        drop(monitor);

        // 获取负载
        let load_avg = self
            .shell
            .execute("cat /proc/loadavg | awk '{print $1,$2,$3}'")
            .map(|r| r.stdout.trim().to_string())
            .unwrap_or_else(|_| "N/A".to_string());

        // 获取监听端口
        let listening_ports = self.get_listening_ports()?;

        // 获取服务状态
        let services = self.get_service_status()?;

        Ok(DiagnosisResult {
            system: SystemDiagnosis {
                hostname: sys_info.hostname,
                os: format!("{} {}", sys_info.os_name, sys_info.os_version),
                kernel: sys_info.kernel_version,
                uptime: format!("{} 秒", sys_info.uptime),
                load_average: load_avg,
            },
            cpu: CpuDiagnosis {
                model: cpu_info.brand,
                cores: cpu_info.cores,
                usage_percent: cpu_info.usage,
                temperature: None,
            },
            memory: MemoryDiagnosis {
                total_mb: mem_info.total / 1024 / 1024,
                used_mb: mem_info.used / 1024 / 1024,
                available_mb: mem_info.available / 1024 / 1024,
                usage_percent: mem_info.usage,
                swap_total_mb: mem_info.swap_total / 1024 / 1024,
                swap_used_mb: mem_info.swap_used / 1024 / 1024,
            },
            disk: disk_info
                .iter()
                .map(|d| DiskDiagnosis {
                    mount_point: d.mount_point.clone(),
                    fs_type: d.fs_type.clone(),
                    total_gb: d.total as f64 / 1024.0 / 1024.0 / 1024.0,
                    used_gb: d.used as f64 / 1024.0 / 1024.0 / 1024.0,
                    available_gb: d.available as f64 / 1024.0 / 1024.0 / 1024.0,
                    usage_percent: d.usage,
                })
                .collect(),
            network: NetworkDiagnosis {
                interfaces: vec!["eth0".to_string()], // TODO: 实际获取
                listening_ports,
                established_connections: 0, // TODO: 实际获取
            },
            services,
        })
    }

    /// 获取性能信息
    pub async fn get_performance_info(&self, _ctx: &ToolContext) -> Result<PerformanceInfo> {
        let monitor = self.monitor.borrow();
        let mem = monitor.get_memory_info();
        drop(monitor);

        let mut monitor = self.monitor.borrow_mut();
        let cpu = monitor.get_cpu_info();
        drop(monitor);

        Ok(PerformanceInfo {
            cpu: CpuPerfInfo {
                usage_percent: cpu.usage,
                user_percent: cpu.usage * 0.6,   // 估算
                system_percent: cpu.usage * 0.3, // 估算
                iowait_percent: cpu.usage * 0.1, // 估算
            },
            memory: MemoryPerfInfo {
                usage_percent: mem.usage,
                cache_mb: 0, // TODO: 实际获取
                buffers_mb: 0,
            },
            disk_io: DiskIoInfo {
                read_bytes_per_sec: 0, // TODO: 实际获取
                write_bytes_per_sec: 0,
            },
            network_io: NetworkIoInfo {
                rx_bytes_per_sec: 0, // TODO: 实际获取
                tx_bytes_per_sec: 0,
            },
        })
    }

    /// 获取安全信息
    pub async fn get_security_info(&self, _ctx: &ToolContext) -> Result<SecurityInfo> {
        let ssh_config = self.get_ssh_config()?;
        let firewall_status = self.get_firewall_status()?;
        let open_ports = self.get_open_ports()?;

        Ok(SecurityInfo {
            ssh_config,
            firewall_status,
            open_ports,
            failed_logins: 0,   // TODO: 从日志解析
            sudo_users: vec![], // TODO: 实际获取
        })
    }
    /// 获取 listening ports。
    fn get_listening_ports(&self) -> Result<Vec<u16>> {
        let result = self
            .shell
            .execute("ss -tln | awk 'NR>1 {print $4}' | cut -d: -f2")?;
        let ports: Vec<u16> = result
            .stdout
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        Ok(ports)
    }
    /// 获取 service status。
    fn get_service_status(&self) -> Result<Vec<ServiceDiagnosis>> {
        let manager = panel_core::ServiceManager::new();
        let services = manager.get_services()?;

        Ok(services
            .into_iter()
            .take(20)
            .map(|s| ServiceDiagnosis {
                name: s.name,
                status: format!("{:?}", s.status),
                enabled: s.enabled,
            })
            .collect())
    }
    /// 获取 ssh config。
    fn get_ssh_config(&self) -> Result<SshConfig> {
        let permit_root = self
            .shell
            .execute("grep -E '^PermitRootLogin' /etc/ssh/sshd_config 2>/dev/null | head -1")
            .map(|r| r.stdout.contains("yes"))
            .unwrap_or(false);

        let password_auth = self
            .shell
            .execute("grep -E '^PasswordAuthentication' /etc/ssh/sshd_config 2>/dev/null | head -1")
            .map(|r| r.stdout.contains("yes"))
            .unwrap_or(true);

        let port = self
            .shell
            .execute(
                "grep -E '^Port' /etc/ssh/sshd_config 2>/dev/null | head -1 | awk '{print $2}'",
            )
            .map(|r| r.stdout.trim().parse().unwrap_or(22))
            .unwrap_or(22);

        Ok(SshConfig {
            permit_root_login: permit_root,
            password_auth,
            port,
        })
    }
    /// 获取 firewall status。
    fn get_firewall_status(&self) -> Result<String> {
        if let Ok(r) = self.shell.execute("ufw status 2>/dev/null | head -1") {
            if !r.stdout.is_empty() {
                return Ok(r.stdout.trim().to_string());
            }
        }

        if let Ok(r) = self.shell.execute("firewall-cmd --state 2>/dev/null") {
            if !r.stdout.is_empty() {
                return Ok(format!("firewalld: {}", r.stdout.trim()));
            }
        }

        Ok("Unknown".to_string())
    }
    /// 获取 open ports。
    fn get_open_ports(&self) -> Result<Vec<PortInfo>> {
        let result = self
            .shell
            .execute("ss -tlnp 2>/dev/null | awk 'NR>1 {print $4, $6}'")?;

        let ports: Vec<PortInfo> = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let port_str = parts[0].split(':').next_back()?;
                    let port: u16 = port_str.parse().ok()?;
                    let service = parts
                        .get(1)
                        .and_then(|s| s.split(',').next())
                        .unwrap_or("unknown")
                        .to_string();

                    Some(PortInfo {
                        port,
                        protocol: "tcp".to_string(),
                        service,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(ports)
    }
}

impl Default for DiagnosticTool {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}
