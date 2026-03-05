//! AI Agents 模块

mod advisor;
mod installer;

pub use advisor::AdvisorAgent;
pub use installer::{AgentResponse, InstallMode, InstallerAgent, UrlInstallReport};
