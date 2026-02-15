//! 服务管理面板组件

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use panel_core::ServiceManager;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

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

        let block = Block::default()
            .title(" 服务管理 (r:刷新 s:启动 t:停止 R:重启) ")
            .borders(Borders::ALL);

        if self.loading {
            let paragraph = Paragraph::new("加载中...").block(block);
            f.render_widget(paragraph, area);
            return;
        }

        if let Some(ref error) = self.error {
            let paragraph = Paragraph::new(format!("错误: {}", error))
                .style(Style::default().fg(Color::Red))
                .block(block);
            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .services
            .iter()
            .enumerate()
            .map(|(i, service)| {
                let status_color = match service.status {
                    panel_core::ServiceStatus::Running => Color::Green,
                    panel_core::ServiceStatus::Stopped => Color::Gray,
                    panel_core::ServiceStatus::Failed => Color::Red,
                    panel_core::ServiceStatus::Loading => Color::Yellow,
                    panel_core::ServiceStatus::Unknown => Color::White,
                };

                let status_text = match service.status {
                    panel_core::ServiceStatus::Running => "●",
                    panel_core::ServiceStatus::Stopped => "○",
                    panel_core::ServiceStatus::Failed => "✗",
                    panel_core::ServiceStatus::Loading => "◐",
                    panel_core::ServiceStatus::Unknown => "?",
                };

                let enabled_text = if service.enabled { "[启用]" } else { "[禁用]" };

                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(format!(
                    " {} {} {} {}",
                    status_text, service.name, enabled_text, service.description
                ))
                .style(Style::default().fg(status_color).bg(if i == self.selected_index {
                    Color::DarkGray
                } else {
                    Color::Reset
                }))
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }
}

impl Default for ServicesPanel {
    fn default() -> Self {
        Self::new()
    }
}
