//! 设置页面组件
//!
//! 提供应用设置界面，包括 AI 配置等

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::theme::Theme;

/// 设置项
#[derive(Debug, Clone)]
pub struct SettingItem {
    /// 设置名称
    pub name: String,
    /// 设置描述
    pub description: String,
    /// 当前值
    pub value: String,
    /// 设置类型
    pub setting_type: SettingType,
}

/// 设置类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingType {
    /// 文本输入
    Text,
    /// 选择项
    Select,
    /// 布尔值
    Boolean,
}

/// 设置页面组件
pub struct SettingsPanel {
    /// 设置项列表
    settings: Vec<SettingItem>,
    /// 选中索引
    selected_index: usize,
    /// 是否正在编辑
    is_editing: bool,
    /// 编辑缓冲区
    edit_buffer: String,
}

impl SettingsPanel {
    /// 创建新的设置页面
    pub fn new() -> Self {
        Self {
            settings: Self::default_settings(),
            selected_index: 0,
            is_editing: false,
            edit_buffer: String::new(),
        }
    }

    /// 默认设置项
    fn default_settings() -> Vec<SettingItem> {
        vec![
            SettingItem {
                name: "AI Provider".to_string(),
                description: "AI 服务提供商".to_string(),
                value: "Claude".to_string(),
                setting_type: SettingType::Select,
            },
            SettingItem {
                name: "API Gateway".to_string(),
                description: "API 网关地址".to_string(),
                value: "https://api.anthropic.com".to_string(),
                setting_type: SettingType::Text,
            },
            SettingItem {
                name: "Model".to_string(),
                description: "使用的模型".to_string(),
                value: "claude-3-opus".to_string(),
                setting_type: SettingType::Select,
            },
            SettingItem {
                name: "Temperature".to_string(),
                description: "生成温度 (0.0-1.0)".to_string(),
                value: "0.7".to_string(),
                setting_type: SettingType::Text,
            },
            SettingItem {
                name: "Enable Debug".to_string(),
                description: "启用调试模式".to_string(),
                value: "否".to_string(),
                setting_type: SettingType::Boolean,
            },
        ]
    }

    /// 处理按键
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.is_editing {
            match key.code {
                KeyCode::Char(c) => {
                    self.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                }
                KeyCode::Enter => {
                    // 保存编辑
                    if let Some(setting) = self.settings.get_mut(self.selected_index) {
                        setting.value = self.edit_buffer.clone();
                    }
                    self.is_editing = false;
                }
                KeyCode::Esc => {
                    self.is_editing = false;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected_index < self.settings.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(setting) = self.settings.get(self.selected_index) {
                        match setting.setting_type {
                            SettingType::Text => {
                                self.is_editing = true;
                                self.edit_buffer = setting.value.clone();
                            }
                            SettingType::Boolean => {
                                if let Some(s) = self.settings.get_mut(self.selected_index) {
                                    s.value = if s.value == "是" { "否" } else { "是" }.to_string();
                                }
                            }
                            SettingType::Select => {
                                // TODO: 实现选择菜单
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// 绘制设置页面
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(0),    // 设置列表
                Constraint::Length(3), // 状态/帮助
            ])
            .split(area);

        // 标题
        let title = Paragraph::new(" 设置 ")
            .style(Theme::title())
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Theme::border()),
            );

        f.render_widget(title, chunks[0]);

        // 设置列表
        self.draw_settings_list(f, chunks[1]);

        // 状态栏
        self.draw_status_bar(f, chunks[2]);
    }

    fn draw_settings_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .settings
            .iter()
            .enumerate()
            .map(|(i, setting)| {
                let style = if i == self.selected_index {
                    Theme::selected_highlight()
                } else {
                    Theme::text()
                };

                let content = format!(
                    " {} {:20} {}",
                    if i == self.selected_index { "►" } else { " " },
                    setting.name,
                    if self.is_editing && i == self.selected_index {
                        format!("{}_", self.edit_buffer)
                    } else {
                        setting.value.clone()
                    }
                );

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::border())
                .title(" 设置项 "),
        );

        f.render_widget(list, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let help_text = if self.is_editing {
            " Enter:保存 | Esc:取消 "
        } else {
            " ↑↓:选择 | Enter:编辑 | Esc:返回 "
        };

        let description = if let Some(setting) = self.settings.get(self.selected_index) {
            &setting.description
        } else {
            ""
        };

        let text = format!(" {} | {}", description, help_text);

        let paragraph = Paragraph::new(text).style(Theme::status_bar());

        f.render_widget(paragraph, area);
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}
