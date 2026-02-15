//! Panel TUI - 终端用户界面模块
//!
//! 提供交互式的命令行操作体验

pub mod app;
pub mod ui;

pub use app::{App, AppMode, AppResult};
pub use ui::dashboard::Dashboard;
pub use ui::wizard::InstallWizard;
pub use ui::services::ServicesPanel;
pub use ui::ai_chat::AiChatPanel;

pub mod prelude {
    pub use crate::app::{App, AppMode, AppResult, AppState};
    pub use crate::ui::dashboard::Dashboard;
    pub use crate::ui::wizard::InstallWizard;
    pub use crate::ui::services::ServicesPanel;
    pub use crate::ui::ai_chat::AiChatPanel;
}

/// 启动 TUI 应用
pub async fn run_tui() -> anyhow::Result<()> {
    let mut app = App::new()?;
    app.run().await
}

/// 启动安装向导
pub async fn run_wizard() -> anyhow::Result<()> {
    let mut app = App::new()?;
    app.set_mode(app::AppMode::Wizard);
    app.run().await
}

/// 启动 AI 对话
pub async fn run_chat() -> anyhow::Result<()> {
    let mut app = App::new()?;
    app.set_mode(app::AppMode::AiChat);
    app.run().await
}
