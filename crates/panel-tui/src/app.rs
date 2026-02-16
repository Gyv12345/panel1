//! TUI 应用状态机

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    style::Modifier,
};
use std::io;
use std::time::Duration;
use std::cell::RefCell;

use crate::ui::dashboard::Dashboard;
use crate::ui::wizard::InstallWizard;
use crate::ui::services::ServicesPanel;
use crate::ui::ai_chat::AiChatPanel;

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
    /// 退出
    Quit,
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
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) |
                        (KeyModifiers::NONE, KeyCode::Char('q')) => {
                            self.state.should_quit = true;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('?')) => {
                            self.state.show_help = !self.state.show_help;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('1')) => {
                            self.state.mode = AppMode::Dashboard;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('2')) => {
                            self.state.mode = AppMode::Services;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('3')) => {
                            self.state.mode = AppMode::Wizard;
                        }
                        (KeyModifiers::NONE, KeyCode::Char('4')) => {
                            self.state.mode = AppMode::AiChat;
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
    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        match self.state.mode {
            AppMode::Dashboard => {
                self.dashboard.handle_key(key);
            }
            AppMode::Services => {
                self.services_panel.borrow_mut().handle_key(key, &self.service_manager)?;
            }
            AppMode::Wizard => {
                self.wizard.borrow_mut().handle_key(key).await?;
            }
            AppMode::AiChat => {
                self.ai_chat.borrow_mut().handle_key(key).await?;
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
                Constraint::Length(3),  // 标题栏
                Constraint::Min(0),     // 主内容
                Constraint::Length(1),  // 状态栏
            ])
            .split(f.area());

        // 绘制标题栏
        self.draw_title_bar(f, chunks[0]);

        // 根据模式绘制主内容
        match self.state.mode {
            AppMode::Dashboard => {
                self.dashboard.draw(f, chunks[1], &self.monitor);
            }
            AppMode::Services => {
                self.services_panel.borrow_mut().draw(f, chunks[1], &self.service_manager);
            }
            AppMode::Wizard => {
                self.wizard.borrow().draw(f, chunks[1]);
            }
            AppMode::AiChat => {
                self.ai_chat.borrow().draw(f, chunks[1]);
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

    /// 绘制标题栏
    fn draw_title_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::style::{Style, Color};

        let title = Paragraph::new(" Panel1 - Linux 服务器管理面板 ")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));

        f.render_widget(title, area);
    }

    /// 绘制状态栏
    fn draw_status_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::Paragraph;
        use ratatui::style::{Style, Color};

        let mode_str = match self.state.mode {
            AppMode::Dashboard => "仪表盘",
            AppMode::Services => "服务管理",
            AppMode::Wizard => "安装向导",
            AppMode::AiChat => "AI 对话",
            AppMode::Quit => "退出",
        };

        let status = format!(
            " {} | 1:仪表盘 2:服务 3:安装 4:AI | ?:帮助 q:退出 ",
            mode_str
        );

        let paragraph = Paragraph::new(status)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));

        f.render_widget(paragraph, area);
    }

    /// 绘制帮助覆盖层
    fn draw_help_overlay(&self, f: &mut ratatui::Frame) {
        use ratatui::widgets::{Block, Borders, Paragraph, Clear};
        use ratatui::style::{Style, Color};
        

        let area = centered_rect(60, 50, f.area());
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" 帮助 ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let help_text = r#"
快捷键说明:

全局:
  1 - 仪表盘
  2 - 服务管理
  3 - 安装向导
  4 - AI 对话
  ? - 显示/隐藏帮助
  q - 退出

仪表盘:
  ↑/↓ - 滚动
  r - 刷新

服务管理:
  ↑/↓ - 选择服务
  Enter - 查看详情
  s - 启动服务
  t - 停止服务

安装向导:
  Tab - 下一步
  ↑/↓ - 选择选项
  Enter - 确认

AI 对话:
  i - 输入消息
  Enter - 发送
  Esc - 取消输入

按任意键关闭帮助
"#;

        let paragraph = Paragraph::new(help_text)
            .block(block);

        f.render_widget(paragraph, area);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// 计算居中区域
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
