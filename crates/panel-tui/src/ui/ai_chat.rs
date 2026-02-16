//! AI 对话面板组件

use anyhow::Result;
use panel_ai::LlmProvider;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 聊天消息
#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
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
                content: "欢迎使用 Panel1 AI 助手！我可以帮助你：\n- 安装和配置服务\n- 诊断系统问题\n- 提供运维建议\n\n请输入你的问题。".to_string(),
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
            .title(" AI 助手 (i:输入 ↑↓:滚动) ")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .skip(self.scroll_offset)
            .flat_map(|msg| {
                let role_prefix = match msg.role.as_str() {
                    "user" => "你: ",
                    "assistant" => "AI: ",
                    _ => "",
                };

                let style = match msg.role.as_str() {
                    "user" => Style::default().fg(Color::Cyan),
                    "assistant" => Style::default().fg(Color::Green),
                    _ => Style::default().fg(Color::Gray),
                };

                msg.content
                    .lines()
                    .enumerate()
                    .map(move |(i, line)| {
                        let text = if i == 0 {
                            format!("{}{}", role_prefix, line)
                        } else {
                            format!("   {}", line)
                        };
                        ListItem::new(text).style(style)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_input_area(&self, f: &mut Frame, area: Rect) {
        let title = if self.waiting_for_response {
            " 等待响应... "
        } else if self.is_inputting {
            " 输入消息 (Enter:发送 Esc:取消) "
        } else {
            " 按 i 开始输入 "
        };

        let block =
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(if self.is_inputting {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

        let input_text = if self.is_inputting {
            format!("{}_", self.input_buffer)
        } else {
            String::new()
        };

        let paragraph = Paragraph::new(input_text).block(block);
        f.render_widget(paragraph, area);
    }
}

impl Default for AiChatPanel {
    fn default() -> Self {
        Self::new()
    }
}
