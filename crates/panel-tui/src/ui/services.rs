//! 服务管理面板组件
//!
//! 使用 Catppuccin Mocha 主题的现代化服务管理界面

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use panel_core::ServiceManager;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::theme::{CatppuccinMocha, Theme};
use crate::ui::components::status_bar;

/// 服务管理面板组件
pub struct ServicesPanel {
    /// 服务列表
    services: Vec<panel_core::ServiceInfo>,
    /// 选中索引
    selected_index: usize,
    /// 是否正在加载
    loading: bool,
    /// 错误信息
    error: Option<String>,
}

impl ServicesPanel {
    /// 创建新的服务面板
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            selected_index: 0,
            loading: false,
            error: None,
        }
    }

    /// 刷新服务列表
    fn refresh_services(&mut self, manager: &ServiceManager) {
        self.loading = true;
        self.error = None;

        match manager.get_services() {
            Ok(services) => {
                self.services = services;
                if self.selected_index >= self.services.len() && !self.services.is_empty() {
                    self.selected_index = self.services.len() - 1;
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }

        self.loading = false;
    }

    /// 处理按键
    pub fn handle_key(&mut self, key: KeyEvent, manager: &ServiceManager) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.services.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('r') => {
                self.refresh_services(manager);
            }
            KeyCode::Char('s') => {
                if let Some(service) = self.services.get(self.selected_index) {
                    let _ = manager.start(&service.name);
                    self.refresh_services(manager);
                }
            }
            KeyCode::Char('t') => {
                if let Some(service) = self.services.get(self.selected_index) {
                    let _ = manager.stop(&service.name);
                    self.refresh_services(manager);
                }
            }
            KeyCode::Char('R') => {
                if let Some(service) = self.services.get(self.selected_index) {
                    let _ = manager.restart(&service.name);
                    self.refresh_services(manager);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 绘制服务面板
    pub fn draw(&mut self, f: &mut Frame, area: Rect, manager: &ServiceManager) {
        // 如果列表为空，尝试加载
        if self.services.is_empty() && !self.loading && self.error.is_none() {
            self.refresh_services(manager);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // 服务列表
                Constraint::Length(1), // 状态栏
            ])
            .split(area);

        // 绘制服务列表
        self.draw_services_list(f, chunks[0]);

        // 绘制状态栏
        self.draw_status_bar(f, chunks[1]);
    }

    fn draw_services_list(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 服务管理 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        if self.loading {
            let paragraph = Paragraph::new(" 加载中...")
                .style(Theme::subtext())
                .block(block);
            f.render_widget(paragraph, area);
            return;
        }

        if let Some(ref error) = self.error {
            let paragraph = Paragraph::new(format!(" 错误: {}", error))
                .style(Theme::error())
                .block(block);
            f.render_widget(paragraph, area);
            return;
        }

        // 按状态分组
        let (running, stopped): (Vec<_>, Vec<_>) = self
            .services
            .iter()
            .enumerate()
            .partition(|(_, s)| matches!(s.status, panel_core::ServiceStatus::Running));

        let mut items = Vec::new();

        // 运行中的服务
        if !running.is_empty() {
            items.push(
                ListItem::new(format!(" ── 运行中 ({}) ──", running.len())).style(Theme::success()),
            );
            for (i, service) in &running {
                items.push(self.create_service_item(*i, service));
            }
        }

        // 已停止的服务
        if !stopped.is_empty() {
            items.push(
                ListItem::new(format!(" ── 已停止 ({}) ──", stopped.len())).style(Theme::subtext()),
            );
            for (i, service) in &stopped {
                items.push(self.create_service_item(*i, service));
            }
        }

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn create_service_item(
        &self,
        index: usize,
        service: &panel_core::ServiceInfo,
    ) -> ListItem<'static> {
        let status_color = match service.status {
            panel_core::ServiceStatus::Running => CatppuccinMocha::GREEN,
            panel_core::ServiceStatus::Stopped => CatppuccinMocha::SUBTEXT0,
            panel_core::ServiceStatus::Failed => CatppuccinMocha::RED,
            panel_core::ServiceStatus::Loading => CatppuccinMocha::YELLOW,
            panel_core::ServiceStatus::Unknown => CatppuccinMocha::SUBTEXT1,
        };

        let status_icon = match service.status {
            panel_core::ServiceStatus::Running => "●",
            panel_core::ServiceStatus::Stopped => "○",
            panel_core::ServiceStatus::Failed => "✗",
            panel_core::ServiceStatus::Loading => "◐",
            panel_core::ServiceStatus::Unknown => "?",
        };

        let enabled_text = if service.enabled {
            "[启用]"
        } else {
            "[禁用]"
        };

        let content = format!(
            "  {} {} {} {}",
            status_icon, service.name, enabled_text, service.description
        );

        let style = if index == self.selected_index {
            ratatui::style::Style::default()
                .fg(status_color)
                .bg(CatppuccinMocha::SURFACE1)
        } else {
            ratatui::style::Style::default().fg(status_color)
        };

        ListItem::new(content).style(style)
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let running_count = self
            .services
            .iter()
            .filter(|s| matches!(s.status, panel_core::ServiceStatus::Running))
            .count();
        let total_count = self.services.len();

        status_bar()
            .hint("r", "刷新")
            .hint("s", "启动")
            .hint("t", "停止")
            .hint("R", "重启")
            .extra(vec![format!("运行中: {}/{}", running_count, total_count)])
            .draw(f, area);
    }
}

impl Default for ServicesPanel {
    fn default() -> Self {
        Self::new()
    }
}
