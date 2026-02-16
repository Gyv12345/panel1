//! TUI 应用状态机
//!
//! 使用 Catppuccin Mocha 主题的现代化界面

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::cell::RefCell;
use std::io;
use std::time::Duration;

use crate::theme::Theme;
use crate::ui::ai_chat::AiChatPanel;
use crate::ui::components::status_bar;
use crate::ui::dashboard::Dashboard;
use crate::ui::services::ServicesPanel;
use crate::ui::settings::SettingsPanel;
use crate::ui::wizard::InstallWizard;

/// 应用版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    /// 系统仪表盘
    Dashboard,
    /// 服务管理
    Services,
    /// 安装向导
    Wizard,
    /// AI 对话
    AiChat,
    /// 设置
    Settings,
    /// 退出
    Quit,
}

impl AppMode {
    /// 获取模式名称
    fn name(&self) -> &'static str {
        match self {
            AppMode::Dashboard => "仪表盘",
            AppMode::Services => "服务管理",
            AppMode::Wizard => "安装向导",
            AppMode::AiChat => "AI 对话",
            AppMode::Settings => "设置",
            AppMode::Quit => "退出",
        }
    }

    /// 获取标签索引
    fn tab_index(&self) -> usize {
        match self {
            AppMode::Dashboard => 0,
            AppMode::Services => 1,
            AppMode::Wizard => 2,
            AppMode::AiChat => 3,
            AppMode::Settings => 4,
            AppMode::Quit => 0,
        }
    }

    /// 从数字键获取模式
    fn from_key(key: char) -> Option<Self> {
        match key {
            '1' => Some(AppMode::Dashboard),
            '2' => Some(AppMode::Services),
            '3' => Some(AppMode::Wizard),
            '4' => Some(AppMode::AiChat),
            '5' => Some(AppMode::Settings),
            _ => None,
        }
    }
}

/// 应用状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 当前模式
    pub mode: AppMode,
    /// 是否应该退出
    pub should_quit: bool,
    /// 当前选中的菜单项
    pub selected_menu: usize,
    /// 是否显示帮助
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Dashboard,
            should_quit: false,
            selected_menu: 0,
            show_help: false,
        }
    }
}

/// TUI 应用结果
pub type AppResult<T> = Result<T>;

/// TUI 应用主结构
pub struct App {
    /// 应用状态
    state: AppState,
    /// 系统监控器
    monitor: RefCell<panel_core::SystemMonitor>,
    /// 服务管理器
    service_manager: panel_core::ServiceManager,
    /// 仪表盘组件
    dashboard: Dashboard,
    /// 服务面板组件
    services_panel: RefCell<ServicesPanel>,
    /// 安装向导组件
    wizard: RefCell<InstallWizard>,
    /// AI 对话面板
    ai_chat: RefCell<AiChatPanel>,
    /// 设置面板
    settings_panel: RefCell<SettingsPanel>,
}

impl App {
    /// 创建新的 TUI 应用
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::default(),
            monitor: RefCell::new(panel_core::SystemMonitor::new()),
            service_manager: panel_core::ServiceManager::new(),
            dashboard: Dashboard::new(),
            services_panel: RefCell::new(ServicesPanel::new()),
            wizard: RefCell::new(InstallWizard::new()),
            ai_chat: RefCell::new(AiChatPanel::new()),
            settings_panel: RefCell::new(SettingsPanel::new()),
        })
    }

    /// 设置初始模式
    pub fn set_mode(&mut self, mode: AppMode) {
        self.state.mode = mode;
    }

    /// 运行应用
    pub async fn run(&mut self) -> Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 主循环
        let res = self.run_loop(&mut terminal).await;

        // 恢复终端
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    /// 主循环
    async fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            // 刷新数据
            self.monitor.borrow_mut().refresh();

            // 绘制界面
            terminal.draw(|f| self.ui(f))?;

            // 处理事件
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    match (key.modifiers, key.code) {
                        // 全局快捷键
                        (KeyModifiers::CONTROL, KeyCode::Char('c'))
                        | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                            self.state.should_quit = true;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('?')) => {
                            self.state.show_help = !self.state.show_help;
                        }
                        (KeyModifiers::NONE, KeyCode::Char(c @ '1'..='5')) => {
                            if let Some(mode) = AppMode::from_key(c) {
                                self.state.mode = mode;
                            }
                        }
                        // 模式特定处理
                        _ => self.handle_key_event(key).await?,
                    }
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// 处理按键事件
    #[allow(clippy::await_holding_refcell_ref)]
    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        match self.state.mode {
            AppMode::Dashboard => {
                self.dashboard.handle_key(key);
            }
            AppMode::Services => {
                self.services_panel
                    .borrow_mut()
                    .handle_key(key, &self.service_manager)?;
            }
            AppMode::Wizard => {
                self.wizard.borrow_mut().handle_key(key).await?;
            }
            AppMode::AiChat => {
                self.ai_chat.borrow_mut().handle_key(key).await?;
            }
            AppMode::Settings => {
                self.settings_panel.borrow_mut().handle_key(key);
            }
            AppMode::Quit => {
                self.state.should_quit = true;
            }
        }
        Ok(())
    }

    /// 绘制界面
    fn ui(&self, f: &mut ratatui::Frame) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // 创建主布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标签栏
                Constraint::Min(0),    // 主内容
                Constraint::Length(1), // 状态栏
            ])
            .split(f.area());

        // 绘制标签栏
        self.draw_tab_bar(f, chunks[0]);

        // 根据模式绘制主内容
        match self.state.mode {
            AppMode::Dashboard => {
                self.dashboard.draw(f, chunks[1], &self.monitor);
            }
            AppMode::Services => {
                self.services_panel
                    .borrow_mut()
                    .draw(f, chunks[1], &self.service_manager);
            }
            AppMode::Wizard => {
                self.wizard.borrow().draw(f, chunks[1]);
            }
            AppMode::AiChat => {
                self.ai_chat.borrow().draw(f, chunks[1]);
            }
            AppMode::Settings => {
                self.settings_panel.borrow().draw(f, chunks[1]);
            }
            AppMode::Quit => {}
        }

        // 绘制状态栏
        self.draw_status_bar(f, chunks[2]);

        // 如果显示帮助，绘制帮助覆盖层
        if self.state.show_help {
            self.draw_help_overlay(f);
        }
    }

    /// 绘制标签栏
    fn draw_tab_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::{Block, Borders, Paragraph, Tabs},
        };

        let tabs = ["仪表盘", "服务", "安装", "AI", "设置"];
        let selected = self.state.mode.tab_index();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(15)])
            .split(area);

        // 标签页
        let titles: Vec<String> = tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                if i == selected {
                    format!(" {} ", tab)
                } else {
                    format!("[{}] {} ", i + 1, tab)
                }
            })
            .collect();

        let tab_widget = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Theme::border()),
            )
            .select(selected)
            .style(Theme::tab_inactive())
            .highlight_style(Theme::tab_active());

        f.render_widget(tab_widget, chunks[0]);

        // 版本号
        let version = Paragraph::new(format!(" Panel1 {} ", VERSION))
            .style(Theme::subtext())
            .alignment(ratatui::layout::Alignment::Right);

        f.render_widget(version, chunks[1]);
    }

    /// 绘制状态栏
    fn draw_status_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let m = self.monitor.borrow();
        let mem_info = m.get_memory_info();
        drop(m);

        let mut m = self.monitor.borrow_mut();
        let cpu_info = m.get_cpu_info();
        drop(m);

        let time = chrono::Local::now().format("%H:%M:%S");

        status_bar()
            .mode(self.state.mode.name())
            .hint("?", "帮助")
            .hint("q", "退出")
            .extra(vec![
                format!("CPU: {:.0}%", cpu_info.usage),
                format!("MEM: {:.0}%", mem_info.usage),
                time.to_string(),
            ])
            .draw(f, area);
    }

    /// 绘制帮助覆盖层
    fn draw_help_overlay(&self, f: &mut ratatui::Frame) {
        use ratatui::{
            layout::{Constraint, Layout},
            widgets::{Block, Borders, Clear, Paragraph},
        };

        let area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" 帮助 ")
            .title_style(Theme::title())
            .borders(Borders::ALL)
            .border_style(Theme::border_accent());

        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        // 全局快捷键
        let global_help = " 全局快捷键:";
        let global = Paragraph::new(global_help)
            .style(Theme::accent())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(global, chunks[0]);

        let nav_help = " [1-5] 切换页面  |  [?] 显示/隐藏帮助  |  [q] 退出";
        let nav = Paragraph::new(nav_help)
            .style(Theme::text())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(nav, chunks[1]);

        // 仪表盘
        let dash_title = " 仪表盘:";
        let dash_t = Paragraph::new(dash_title)
            .style(Theme::accent())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(dash_t, chunks[2]);

        let dash_help = "  [↑↓] 滚动  |  [r] 刷新";
        let dash = Paragraph::new(dash_help)
            .style(Theme::text())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(dash, chunks[3]);

        // 服务管理
        let svc_title = " 服务管理:";
        let svc_t = Paragraph::new(svc_title)
            .style(Theme::accent())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(svc_t, chunks[4]);

        let svc_help = "  [↑↓] 选择  |  [s] 启动  |  [t] 停止  |  [R] 重启  |  [r] 刷新";
        let svc = Paragraph::new(svc_help)
            .style(Theme::text())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(svc, chunks[5]);

        // AI 对话
        let ai_help = " AI 对话: [i] 输入  |  [Enter] 发送  |  [Esc] 取消  |  [↑↓] 滚动";
        let ai = Paragraph::new(ai_help)
            .style(Theme::subtext())
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(ai, chunks[6]);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// 计算居中区域
fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Layout};

    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
