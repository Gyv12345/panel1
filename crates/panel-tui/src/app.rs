//! TUI 应用状态机（极简版）
//!
//! 仅保留两个核心区域：服务器监控 + AI 安装 Agent。

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
use crate::ui::ai_installer::AiInstallerPanel;
use crate::ui::components::status_bar;
use crate::ui::dashboard::Dashboard;

/// 应用版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    /// 服务器监控
    Dashboard,
    /// AI 安装 Agent
    AiInstaller,
    /// 退出
    Quit,
}

impl AppMode {
    /// 获取模式名称
    fn name(&self) -> &'static str {
        match self {
            AppMode::Dashboard => "监控",
            AppMode::AiInstaller => "AI 安装",
            AppMode::Quit => "退出",
        }
    }

    /// 获取标签索引
    fn tab_index(&self) -> usize {
        match self {
            AppMode::Dashboard => 0,
            AppMode::AiInstaller => 1,
            AppMode::Quit => 0,
        }
    }

    /// 从数字键获取模式
    fn from_key(key: char) -> Option<Self> {
        match key {
            '1' => Some(AppMode::Dashboard),
            '2' => Some(AppMode::AiInstaller),
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
    /// 是否显示帮助
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Dashboard,
            should_quit: false,
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
    /// 仪表盘组件
    dashboard: Dashboard,
    /// AI 安装 Agent 面板
    ai_installer: RefCell<AiInstallerPanel>,
}

impl App {
    /// 创建新的 TUI 应用
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::default(),
            monitor: RefCell::new(panel_core::SystemMonitor::new()),
            dashboard: Dashboard::new(),
            ai_installer: RefCell::new(AiInstallerPanel::new()),
        })
    }

    /// 设置初始模式
    pub fn set_mode(&mut self, mode: AppMode) {
        self.state.mode = mode;
    }

    /// 运行应用
    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_loop(&mut terminal).await;

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
            self.monitor.borrow_mut().refresh();

            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('c'))
                        | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                            self.state.should_quit = true;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('?')) => {
                            self.state.show_help = !self.state.show_help;
                        }
                        (KeyModifiers::NONE, KeyCode::Char(c @ '1'..='2')) => {
                            if let Some(mode) = AppMode::from_key(c) {
                                self.state.mode = mode;
                            }
                        }
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
            AppMode::Dashboard => self.dashboard.handle_key(key),
            AppMode::AiInstaller => self.ai_installer.borrow_mut().handle_key(key).await?,
            AppMode::Quit => self.state.should_quit = true,
        }
        Ok(())
    }

    /// 绘制界面
    fn ui(&self, f: &mut ratatui::Frame) {
        use ratatui::layout::{Constraint, Direction, Layout};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(f.area());

        self.draw_tab_bar(f, chunks[0]);

        match self.state.mode {
            AppMode::Dashboard => self.dashboard.draw(f, chunks[1], &self.monitor),
            AppMode::AiInstaller => self.ai_installer.borrow().draw(f, chunks[1]),
            AppMode::Quit => {}
        }

        self.draw_status_bar(f, chunks[2]);

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

        let tabs = ["监控", "AI 安装"];
        let selected = self.state.mode.tab_index();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(15)])
            .split(area);

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

        let area = centered_rect(60, 56, f.area());
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
                Constraint::Min(0),
            ])
            .split(inner);

        f.render_widget(
            Paragraph::new(" 全局快捷键:")
                .style(Theme::accent())
                .alignment(ratatui::layout::Alignment::Left),
            chunks[0],
        );

        f.render_widget(
            Paragraph::new(" [1-2] 切换页面  |  [?] 显示/隐藏帮助  |  [q] 退出")
                .style(Theme::text())
                .alignment(ratatui::layout::Alignment::Left),
            chunks[1],
        );

        f.render_widget(
            Paragraph::new(" 监控页: [↑↓] 滚动  |  [r] 刷新")
                .style(Theme::text())
                .alignment(ratatui::layout::Alignment::Left),
            chunks[2],
        );

        f.render_widget(
            Paragraph::new(
                " AI 安装页: 输入 URL + 回车安装  |  [Tab] 切换输入项  |  [m/←→] 切换模式",
            )
            .style(Theme::subtext())
            .alignment(ratatui::layout::Alignment::Left),
            chunks[3],
        );

        f.render_widget(
            Paragraph::new(" 目标: 给 URL 即可安装工具服务")
                .style(Theme::subtext())
                .alignment(ratatui::layout::Alignment::Left),
            chunks[4],
        );
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
