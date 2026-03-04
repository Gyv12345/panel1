//! Panel TUI - 终端用户界面模块
//!
//! 提供交互式的命令行操作体验

pub mod app;
pub mod theme;
pub mod ui;

pub use app::{App, AppMode, AppResult};
pub use ui::ai_installer::AiInstallerPanel;
pub use ui::dashboard::Dashboard;

pub mod prelude {
    pub use crate::app::{App, AppMode, AppResult, AppState};
    pub use crate::theme::{CatppuccinMocha, Theme};
    pub use crate::ui::ai_installer::AiInstallerPanel;
    pub use crate::ui::dashboard::Dashboard;
}

/// 启动 TUI 应用
pub async fn run_tui() -> anyhow::Result<()> {
    let mut app = App::new()?;
    app.run().await
}
