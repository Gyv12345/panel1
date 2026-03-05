//! AI 安装 Agent 面板
//!
//! 输入 URL 后自动安装工具服务，失败时自动重试并记录修复日志。

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use panel_ai::{InstallMode, InstallerAgent, LlmProvider};
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
    Mode,
    Profile,
}

/// AI 安装 Agent 面板
pub struct AiInstallerPanel {
    url_input: String,
    name_input: String,
    install_mode: InstallMode,
    focus: FieldFocus,
    installing: bool,
    logs: Vec<String>,
    installer: InstallerAgent,
    profile_names: Vec<String>,
    active_profile_idx: Option<usize>,
}

impl AiInstallerPanel {
    /// 创建新面板
    pub fn new() -> Self {
        let provider: Arc<dyn LlmProvider> = Arc::new(panel_ai::ClaudeProvider::new());
        let installer = InstallerAgent::new(provider);

        let mut panel = Self {
            url_input: String::new(),
            name_input: String::new(),
            install_mode: InstallMode::Auto,
            focus: FieldFocus::Url,
            installing: false,
            logs: vec![
                "欢迎使用 AI 安装 Agent".to_string(),
                "输入工具 URL 后按 Enter 开始安装".to_string(),
            ],
            installer,
            profile_names: Vec::new(),
            active_profile_idx: None,
        };

        panel.refresh_profiles();
        panel
    }

    /// 刷新本地 profile 缓存。
    fn refresh_profiles(&mut self) {
        match panel_ai::load_ai_store() {
            Ok(store) => {
                self.profile_names = store
                    .profiles
                    .iter()
                    .map(|profile| profile.name.clone())
                    .collect();
                self.active_profile_idx = self
                    .profile_names
                    .iter()
                    .position(|name| *name == store.active_profile);
            }
            Err(err) => {
                self.logs.push(format!("读取 AI profile 失败: {}", err));
                self.profile_names.clear();
                self.active_profile_idx = None;
            }
        }
    }

    /// 返回当前 active profile 显示文本。
    fn active_profile_label(&self) -> String {
        if self.profile_names.is_empty() {
            return "未配置（先运行 panel1 ai seed-presets / panel1 ai config）".to_string();
        }

        let index = self.active_profile_idx.unwrap_or(0);
        let shown = index + 1;
        let total = self.profile_names.len();
        let name = self
            .profile_names
            .get(index)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        format!("{name} ({shown}/{total})")
    }

    /// 切换 profile 并写回配置。
    fn cycle_profile(&mut self, forward: bool) {
        if self.profile_names.is_empty() {
            self.logs
                .push("未检测到任何 profile，请先运行 panel1 ai seed-presets".to_string());
            return;
        }

        if self.profile_names.len() == 1 {
            self.logs.push("当前只有一个 profile，无需切换".to_string());
            return;
        }

        let current = self.active_profile_idx.unwrap_or(0);
        let total = self.profile_names.len();
        let next = if forward {
            (current + 1) % total
        } else {
            (current + total - 1) % total
        };

        let target_name = self.profile_names[next].clone();
        match panel_ai::load_ai_store() {
            Ok(mut store) => {
                if !store.set_active_profile(&target_name) {
                    self.logs
                        .push(format!("切换 profile 失败，未找到: {}", target_name));
                    return;
                }

                if let Err(err) = panel_ai::save_ai_store(&store) {
                    self.logs.push(format!("保存 active profile 失败: {}", err));
                    return;
                }

                self.active_profile_idx = Some(next);
                self.logs
                    .push(format!("已切换 AI profile: {}", target_name));

                // 重新初始化 Provider，使切换立即生效。
                let provider: Arc<dyn LlmProvider> = Arc::new(panel_ai::ClaudeProvider::new());
                self.installer = InstallerAgent::new(provider);
            }
            Err(err) => {
                self.logs.push(format!("加载 AI 配置失败: {}", err));
            }
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
                    FieldFocus::Name => FieldFocus::Mode,
                    FieldFocus::Mode => FieldFocus::Profile,
                    FieldFocus::Profile => FieldFocus::Url,
                };
            }
            KeyCode::Up => {
                self.focus = match self.focus {
                    FieldFocus::Url => FieldFocus::Profile,
                    FieldFocus::Name => FieldFocus::Url,
                    FieldFocus::Mode => FieldFocus::Name,
                    FieldFocus::Profile => FieldFocus::Mode,
                };
            }
            KeyCode::Left => match self.focus {
                FieldFocus::Mode => {
                    self.install_mode = prev_install_mode(self.install_mode);
                }
                FieldFocus::Profile => {
                    self.cycle_profile(false);
                }
                _ => {}
            },
            KeyCode::Right => match self.focus {
                FieldFocus::Mode => {
                    self.install_mode = next_install_mode(self.install_mode);
                }
                FieldFocus::Profile => {
                    self.cycle_profile(true);
                }
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                FieldFocus::Url => {
                    self.url_input.pop();
                }
                FieldFocus::Name => {
                    self.name_input.pop();
                }
                FieldFocus::Mode | FieldFocus::Profile => {}
            },
            KeyCode::Esc => match self.focus {
                FieldFocus::Url => self.url_input.clear(),
                FieldFocus::Name => self.name_input.clear(),
                FieldFocus::Mode => self.install_mode = InstallMode::Auto,
                FieldFocus::Profile => self.refresh_profiles(),
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
                FieldFocus::Mode => {
                    if c == 'm' || c == 'M' {
                        self.install_mode = next_install_mode(self.install_mode);
                    }
                }
                FieldFocus::Profile => {
                    if c == 'p' || c == 'P' {
                        self.cycle_profile(true);
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }

    /// 执行安装流程。
    async fn install(&mut self) -> Result<()> {
        self.installing = true;
        self.logs
            .push(format!("开始安装: {}", self.url_input.trim()));

        let preferred_name = if self.name_input.trim().is_empty() {
            None
        } else {
            Some(self.name_input.trim())
        };

        match self
            .installer
            .install_from_url(self.url_input.trim(), preferred_name, self.install_mode)
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
        self.draw_input_line(
            f,
            chunks[2],
            "安装方案",
            install_mode_label(self.install_mode),
            self.focus == FieldFocus::Mode,
        );
        self.draw_input_line(
            f,
            chunks[3],
            "AI Profile",
            &self.active_profile_label(),
            self.focus == FieldFocus::Profile,
        );
        self.draw_hint(f, chunks[4]);
        self.draw_logs(f, chunks[5]);
    }

    /// 绘制单行输入。
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

    /// 绘制快捷键提示。
    fn draw_hint(&self, f: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled("[Tab/↑↓] ", Theme::accent()),
            Span::styled("切换输入  ", Theme::text()),
            Span::styled("[Enter] ", Theme::accent()),
            Span::styled("开始安装  ", Theme::text()),
            Span::styled("[m/←→] ", Theme::accent()),
            Span::styled("切换安装方案/模型档  ", Theme::text()),
            Span::styled("[p] ", Theme::accent()),
            Span::styled("切换 AI Profile", Theme::text()),
        ];

        if self.installing {
            spans.push(Span::styled("  安装中...", Theme::warning()));
        }

        let paragraph = Paragraph::new(Line::from(spans)).style(Theme::subtext());
        f.render_widget(paragraph, area);
    }

    /// 绘制日志列表。
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
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}

/// 返回安装模式显示文本。
fn install_mode_label(mode: InstallMode) -> &'static str {
    match mode {
        InstallMode::Auto => "auto（自动检测依赖）",
        InstallMode::Panel1 => "panel1（二进制直装）",
        InstallMode::Docker => "docker（强制 Docker 方案）",
    }
}

/// 切换到下一个安装模式。
fn next_install_mode(mode: InstallMode) -> InstallMode {
    match mode {
        InstallMode::Auto => InstallMode::Panel1,
        InstallMode::Panel1 => InstallMode::Docker,
        InstallMode::Docker => InstallMode::Auto,
    }
}

/// 切换到上一个安装模式。
fn prev_install_mode(mode: InstallMode) -> InstallMode {
    match mode {
        InstallMode::Auto => InstallMode::Docker,
        InstallMode::Panel1 => InstallMode::Auto,
        InstallMode::Docker => InstallMode::Panel1,
    }
}
