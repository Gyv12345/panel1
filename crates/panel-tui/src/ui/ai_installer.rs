//! AI 安装 Agent 面板
//!
//! 输入 URL 后自动安装工具服务，失败时自动重试并记录修复日志。

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use panel_ai::{InstallerAgent, LlmProvider};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldFocus {
    Url,
    Name,
}

/// AI 安装 Agent 面板
pub struct AiInstallerPanel {
    url_input: String,
    name_input: String,
    focus: FieldFocus,
    installing: bool,
    logs: Vec<String>,
    installer: InstallerAgent,
}

impl AiInstallerPanel {
    /// 创建新面板
    pub fn new() -> Self {
        let provider: Arc<dyn LlmProvider> = Arc::new(panel_ai::ClaudeProvider::new());

        Self {
            url_input: String::new(),
            name_input: String::new(),
            focus: FieldFocus::Url,
            installing: false,
            logs: vec![
                "欢迎使用 AI 安装 Agent".to_string(),
                "输入工具 URL 后按 Enter 开始安装".to_string(),
            ],
            installer: InstallerAgent::new(provider),
        }
    }

    /// 处理按键
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.installing {
            return Ok(());
        }

        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                self.focus = match self.focus {
                    FieldFocus::Url => FieldFocus::Name,
                    FieldFocus::Name => FieldFocus::Url,
                };
            }
            KeyCode::Up => {
                self.focus = match self.focus {
                    FieldFocus::Url => FieldFocus::Name,
                    FieldFocus::Name => FieldFocus::Url,
                };
            }
            KeyCode::Backspace => match self.focus {
                FieldFocus::Url => {
                    self.url_input.pop();
                }
                FieldFocus::Name => {
                    self.name_input.pop();
                }
            },
            KeyCode::Esc => match self.focus {
                FieldFocus::Url => self.url_input.clear(),
                FieldFocus::Name => self.name_input.clear(),
            },
            KeyCode::Enter => {
                if self.url_input.trim().is_empty() {
                    self.logs.push("请输入 URL".to_string());
                    return Ok(());
                }
                self.install().await?;
            }
            KeyCode::Char(c) => match self.focus {
                FieldFocus::Url => self.url_input.push(c),
                FieldFocus::Name => self.name_input.push(c),
            },
            _ => {}
        }

        Ok(())
    }

    async fn install(&mut self) -> Result<()> {
        self.installing = true;
        self.logs
            .push(format!("开始安装: {}", self.url_input.trim().to_string()));

        let preferred_name = if self.name_input.trim().is_empty() {
            None
        } else {
            Some(self.name_input.trim())
        };

        match self
            .installer
            .install_from_url(self.url_input.trim(), preferred_name)
            .await
        {
            Ok(report) => {
                self.logs.extend(report.logs);
                if report.success {
                    self.logs.push(format!(
                        "安装完成: {}",
                        report.service_name.unwrap_or_else(|| "unknown".to_string())
                    ));
                    if let Some(path) = report.binary_path {
                        self.logs.push(format!("二进制路径: {}", path));
                    }
                } else {
                    self.logs.push(
                        report
                            .error
                            .unwrap_or_else(|| "安装失败，自动修复未成功".to_string()),
                    );
                }
            }
            Err(err) => {
                self.logs.push(format!("安装失败: {}", err));
            }
        }

        self.installing = false;
        Ok(())
    }

    /// 绘制面板
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(area);

        self.draw_input_line(
            f,
            chunks[0],
            "URL",
            &self.url_input,
            self.focus == FieldFocus::Url,
        );
        self.draw_input_line(
            f,
            chunks[1],
            "服务名(可选)",
            &self.name_input,
            self.focus == FieldFocus::Name,
        );
        self.draw_hint(f, chunks[2]);
        self.draw_logs(f, chunks[3]);
    }

    fn draw_input_line(&self, f: &mut Frame, area: Rect, title: &str, value: &str, focused: bool) {
        let block = Block::default()
            .title(format!(" {} ", title))
            .title_style(if focused {
                Theme::accent()
            } else {
                Theme::subtext()
            })
            .borders(Borders::ALL)
            .border_style(if focused {
                Theme::border_accent()
            } else {
                Theme::border()
            });

        let display = if focused {
            format!("{}_", value)
        } else {
            value.to_string()
        };

        let paragraph = Paragraph::new(display)
            .style(if focused {
                Theme::input_focused()
            } else {
                Theme::input()
            })
            .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_hint(&self, f: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled("[Tab/↑↓] ", Theme::accent()),
            Span::styled("切换输入  ", Theme::text()),
            Span::styled("[Enter] ", Theme::accent()),
            Span::styled("开始安装  ", Theme::text()),
            Span::styled("[Esc] ", Theme::accent()),
            Span::styled("清空当前输入", Theme::text()),
        ];

        if self.installing {
            spans.push(Span::styled("  安装中...", Theme::warning()));
        }

        let paragraph = Paragraph::new(Line::from(spans)).style(Theme::subtext());
        f.render_widget(paragraph, area);
    }

    fn draw_logs(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Agent 日志 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let items: Vec<ListItem> = self
            .logs
            .iter()
            .rev()
            .take(50)
            .rev()
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let list = List::new(items).block(block).style(Theme::text());
        f.render_widget(list, area);
    }
}

impl Default for AiInstallerPanel {
    fn default() -> Self {
        Self::new()
    }
}
