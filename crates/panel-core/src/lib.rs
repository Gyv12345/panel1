//! Panel Core - Linux 系统交互核心库
//!
//! 提供系统信息获取、进程管理、服务管理、网络配置等功能

pub mod network;
pub mod process;
pub mod service;
pub mod system;

pub use network::*;
pub use process::*;
pub use service::*;
pub use system::*;

pub mod prelude {
    pub use crate::network::{ListeningPort, NetworkInterface, NetworkManager, NetworkTraffic};
    pub use crate::process::{ProcessInfo, ProcessManager};
    pub use crate::service::{ServiceInfo, ServiceManager, ServiceStatus};
    pub use crate::system::{CpuInfo, DiskInfo, MemoryInfo, SystemInfo, SystemMonitor};
}
