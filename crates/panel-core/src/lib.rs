//! Panel Core - Linux 系统交互核心库
//!
//! 提供系统信息获取、进程管理、服务管理、网络配置等功能

pub mod system;
pub mod process;
pub mod service;
pub mod network;

pub use system::*;
pub use process::*;
pub use service::*;
pub use network::*;

pub mod prelude {
    pub use crate::system::{SystemInfo, CpuInfo, MemoryInfo, DiskInfo, SystemMonitor};
    pub use crate::process::{ProcessInfo, ProcessManager};
    pub use crate::service::{ServiceInfo, ServiceManager, ServiceStatus};
    pub use crate::network::{NetworkInterface, NetworkTraffic, NetworkManager, ListeningPort};
}
