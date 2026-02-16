//! UI 组件模块
//!
//! 提供可复用的 UI 组件

pub mod card;
pub mod progress;
pub mod status_bar;
pub mod tabs;

pub use card::{card, info_card, Card, CardStyle, InfoCard};
pub use progress::{labeled_progress, progress, resource_usage, LabeledProgress, ProgressBar};
pub use status_bar::{status_bar, StatusBar};
pub use tabs::{TabBar, APP_TABS};

// Tab 类型别名
pub type Tab = tabs::Tab;
