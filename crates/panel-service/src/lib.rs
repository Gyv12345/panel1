//! Panel Service - 混合模式服务管理模块
//!
//! 提供系统级（systemd）和 Panel1 托管二进制的统一服务管理

pub mod binary;
pub mod manager;
pub mod registry;
pub mod systemd;

pub use binary::{BinaryBackend, BinaryConfig, ProcessGuard, UrlInstallMode};
pub use manager::{ManagedService, ServiceManager, ServiceMode};
pub use registry::{
    Artifact, Checksum, DownloadManager, DownloadProgress, InstallConfig, PackageCategory,
    PackageConfig, PackageIndex, PackageRegistry, PackageSummary, PackageVersion, RegistryConfig,
};
pub use systemd::SystemdBackend;

pub mod prelude {
    pub use crate::binary::{BinaryBackend, BinaryConfig, ProcessGuard, UrlInstallMode};
    pub use crate::manager::{ManagedService, ServiceManager, ServiceMode};
}
