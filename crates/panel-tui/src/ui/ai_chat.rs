//! AI 对话面板组件
//!
//! 使用 Catppuccin Mocha 主题的现代化 AI 对话界面

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use panel_ai::LlmProvider;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::theme::Theme;

/// 聊天消息
#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

impl ChatMessage {
    /// 获取样式
    fn style(&self) -> ratatui::style::Style {
        match self.role.as_str() {
            "user" => Theme::accent(),
            "assistant" => Theme::success(),
            _ => Theme::subtext(),
        }
    }

    /// 获取前缀
    fn prefix(&self) -> &'static str {
        match self.role.as_str() {
            "user" => "你",
            "assistant" => "AI",
            _ => "系统",
        }
    }

    /// 获取图标
    fn icon(&self) -> &'static str {
        match self.role.as_str() {
            "user" => "◈",
            "assistant" => "◆",
            _ => "○",
        }
    }
}

/// AI 对话面板组件
pub struct AiChatPanel {
    /// 聊天历史
    messages: Vec<ChatMessage>,
    /// 输入缓冲区
    input_buffer: String,
    /// 是否正在输入
    is_inputting: bool,
    /// 滚动偏移
    scroll_offset: usize,
    /// 是否正在等待响应
    waiting_for_response: bool,
    /// AI Provider
    provider: panel_ai::ClaudeProvider,
}

impl AiChatPanel {
    /// 创建新的 AI 对话面板
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "欢迎使用 Panel1 AI 助手！\n\n我可以帮助你：\n• 安装和配置服务\n• 诊断系统问题\n• 提供运维建议\n\n按 i 开始输入你的问题。".to_string(),
            }],
            input_buffer: String::new(),
            is_inputting: false,
            scroll_offset: 0,
            waiting_for_response: false,
            provider: panel_ai::ClaudeProvider::new(),
        }
    }

    /// 处理按键
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.waiting_for_response {
            return Ok(());
        }

        if self.is_inputting {
            match key.code {
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        self.send_message().await?;
                    }
                }
                KeyCode::Esc => {
                    self.is_inputting = false;
                    self.input_buffer.clear();
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('i') => {
                    self.is_inputting = true;
                }
                KeyCode::Up => {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    self.scroll_offset += 1;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 发送消息
    async fn send_message(&mut self) -> Result<()> {
        let user_message = self.input_buffer.clone();
        self.input_buffer.clear();
        self.is_inputting = false;

        // 添加用户消息
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_message.clone(),
        });

        self.waiting_for_response = true;

        // 使用真实的 AI Provider
        let response = match self.provider.send(&user_message).await {
            Ok(resp) => resp.content,
            Err(e) => format!("AI 响应错误: {}", e),
        };

        // 添加 AI 响应
        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response,
        });

        self.waiting_for_response = false;
        self.scroll_offset = self.messages.len().saturating_sub(1);

        Ok(())
    }

    /// 绘制 AI 对话面板
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // 聊天区域
                Constraint::Length(3), // 输入区域
            ])
            .split(area);

        // 聊天区域
        self.draw_chat_area(f, chunks[0]);

        // 输入区域
        self.draw_input_area(f, chunks[1]);
    }

    fn draw_chat_area(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" AI 助手 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .skip(self.scroll_offset)
            .flat_map(|msg| {
                let style = msg.style();
                let prefix = msg.prefix();
                let icon = msg.icon();

                msg.content
                    .lines()
                    .enumerate()
                    .map(move |(i, line)| {
                        let spans = if i == 0 {
                            vec![
                                Span::styled(icon, style),
                                Span::raw(" "),
                                Span::styled(
                                    prefix,
                                    style.add_modifier(ratatui::style::Modifier::BOLD),
                                ),
                                Span::raw(": "),
                                Span::styled(line, Theme::text()),
                            ]
                        } else {
                            vec![Span::raw("    "), Span::styled(line, Theme::text())]
                        };
                        ListItem::new(Line::from(spans))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_input_area(&self, f: &mut Frame, area: Rect) {
        let (title, border_style) = if self.waiting_for_response {
            (" 等待响应... ", Theme::border_warning())
        } else if self.is_inputting {
            (" 输入消息 ", Theme::border_accent())
        } else {
            (" 按 i 开始输入 ", Theme::border())
        };

        let block = Block::default()
            .title(title)
            .title_style(if self.is_inputting {
                Theme::accent()
            } else {
                Theme::subtext()
            })
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        f.render_widget(block, area);

        // 输入内容
        if self.is_inputting {
            let input_text = format!("{}_", self.input_buffer);
            let paragraph = Paragraph::new(input_text).style(Theme::input_focused());
            f.render_widget(paragraph, inner);
        } else if self.waiting_for_response {
            let paragraph = Paragraph::new("AI 正在思考...").style(Theme::warning());
            f.render_widget(paragraph, inner);
        } else {
            let hint = "按 [i] 开始输入 | [↑↓] 滚动";
            let paragraph = Paragraph::new(hint).style(Theme::subtext());
            f.render_widget(paragraph, inner);
        }
    }
}

impl Default for AiChatPanel {
    fn default() -> Self {
        Self::new()
    }
}
