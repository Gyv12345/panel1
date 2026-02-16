//! 主题模块
//!
//! 提供 Catppuccin Mocha 配色方案和统一的样式系统

pub mod colors;
#[allow(clippy::module_inception)]
pub mod theme;

pub use colors::{CatppuccinMocha, ServiceStatusColor};
pub use theme::{Colors, Theme};
