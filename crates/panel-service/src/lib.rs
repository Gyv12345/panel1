//! Panel Service - 混合模式服务管理模块
//!
//! 提供系统级（systemd）和 Panel1 托管二进制的统一服务管理

pub mod binary;
pub mod manager;
pub mod systemd;
pub mod templates;

pub use binary::{BinaryBackend, BinaryConfig, ProcessGuard};
pub use manager::{ManagedService, ServiceManager, ServiceMode};
pub use systemd::SystemdBackend;
pub use templates::{ServiceTemplate, TemplateRegistry};

pub mod prelude {
    pub use crate::binary::{BinaryBackend, BinaryConfig, ProcessGuard};
    pub use crate::manager::{ManagedService, ServiceManager, ServiceMode};
    pub use crate::templates::{ServiceTemplate, TemplateRegistry};
}
