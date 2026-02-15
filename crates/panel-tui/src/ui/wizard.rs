//! 安装向导组件

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use panel_service::TemplateRegistry;

/// 向导步骤
#[derive(Debug, Clone, Copy, PartialEq)]
enum WizardStep {
    SelectService,
    SelectVersion,
    SelectMode,
    Configure,
    Confirm,
    Installing,
    Done,
}

/// 安装向导组件
pub struct InstallWizard {
    /// 当前步骤
    current_step: WizardStep,
    /// 模板注册表
    templates: TemplateRegistry,
    /// 选中的服务
    selected_service: Option<String>,
    /// 选中的版本
    selected_version: Option<String>,
    /// 选中的模式
    selected_mode: usize,
    /// 配置端口
    config_port: String,
    /// 列表选择索引
    list_index: usize,
    /// 安装消息
    install_message: String,
}

impl InstallWizard {
    /// 创建新的安装向导
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::SelectService,
            templates: TemplateRegistry::new(),
            selected_service: None,
            selected_version: None,
            selected_mode: 0,
            config_port: String::new(),
            list_index: 0,
            install_message: String::new(),
        }
    }

    /// 处理按键
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.current_step {
            WizardStep::SelectService => {
                match key.code {
                    KeyCode::Up => {
                        if self.list_index > 0 {
                            self.list_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let templates = self.templates.list();
                        if self.list_index < templates.len().saturating_sub(1) {
                            self.list_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let templates = self.templates.list();
                        if let Some(template) = templates.get(self.list_index) {
                            self.selected_service = Some(template.id.clone());
                            self.selected_version = Some(template.default_version.clone());
                            self.current_step = WizardStep::SelectVersion;
                            self.list_index = 0;
                        }
                    }
                    _ => {}
                }
            }
            WizardStep::SelectVersion => {
                match key.code {
                    KeyCode::Up => {
                        if self.list_index > 0 {
                            self.list_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if let Some(svc_id) = &self.selected_service {
                            if let Some(template) = self.templates.get(svc_id) {
                                if self.list_index < template.available_versions.len().saturating_sub(1) {
                                    self.list_index += 1;
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(svc_id) = &self.selected_service {
                            if let Some(template) = self.templates.get(svc_id) {
                                if let Some(version) = template.available_versions.get(self.list_index) {
                                    self.selected_version = Some(version.clone());
                                    self.current_step = WizardStep::SelectMode;
                                    self.list_index = 0;
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.current_step = WizardStep::SelectService;
                        self.list_index = 0;
                    }
                    _ => {}
                }
            }
            WizardStep::SelectMode => {
                match key.code {
                    KeyCode::Up => {
                        if self.list_index > 0 {
                            self.list_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.list_index < 2 {
                            self.list_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.selected_mode = self.list_index;
                        self.current_step = WizardStep::Configure;
                        // 设置默认端口
                        if let Some(svc_id) = &self.selected_service {
                            if let Some(template) = self.templates.get(svc_id) {
                                self.config_port = template.default_port.to_string();
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.current_step = WizardStep::SelectVersion;
                        self.list_index = 0;
                    }
                    _ => {}
                }
            }
            WizardStep::Configure => {
                match key.code {
                    KeyCode::Char(c) => {
                        if c.is_ascii_digit() {
                            self.config_port.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.config_port.pop();
                    }
                    KeyCode::Enter => {
                        self.current_step = WizardStep::Confirm;
                    }
                    KeyCode::Esc => {
                        self.current_step = WizardStep::SelectMode;
                    }
                    _ => {}
                }
            }
            WizardStep::Confirm => {
                match key.code {
                    KeyCode::Enter => {
                        self.current_step = WizardStep::Installing;
                        // 模拟安装
                        self.install_message = format!(
                            "正在安装 {} {}...\n\n这可能需要几分钟时间。",
                            self.selected_service.as_deref().unwrap_or("unknown"),
                            self.selected_version.as_deref().unwrap_or("latest")
                        );
                        // TODO: 实际安装逻辑
                    }
                    KeyCode::Esc => {
                        self.current_step = WizardStep::Configure;
                    }
                    _ => {}
                }
            }
            WizardStep::Installing => {
                // 安装中，忽略输入
            }
            WizardStep::Done => {
                if key.code == KeyCode::Enter {
                    // 返回第一步
                    self.reset();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 重置向导
    fn reset(&mut self) {
        self.current_step = WizardStep::SelectService;
        self.selected_service = None;
        self.selected_version = None;
        self.selected_mode = 0;
        self.config_port = String::new();
        self.list_index = 0;
        self.install_message = String::new();
    }

    /// 绘制向导
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        match self.current_step {
            WizardStep::SelectService => self.draw_service_selection(f, area),
            WizardStep::SelectVersion => self.draw_version_selection(f, area),
            WizardStep::SelectMode => self.draw_mode_selection(f, area),
            WizardStep::Configure => self.draw_configuration(f, area),
            WizardStep::Confirm => self.draw_confirmation(f, area),
            WizardStep::Installing => self.draw_installing(f, area),
            WizardStep::Done => self.draw_done(f, area),
        }
    }

    fn draw_service_selection(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 步骤 1/5: 选择要安装的服务 ")
            .borders(Borders::ALL);

        let templates = self.templates.list();
        let items: Vec<ListItem> = templates
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.list_index {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(format!(" {} - {}", t.name, t.description)).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_version_selection(&self, f: &mut Frame, area: Rect) {
        let service_name = self.selected_service.as_deref().unwrap_or("unknown");
        let block = Block::default()
            .title(format!(" 步骤 2/5: 选择 {} 版本 ", service_name))
            .borders(Borders::ALL);

        if let Some(template) = self.templates.get(service_name) {
            let items: Vec<ListItem> = template
                .available_versions
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let style = if i == self.list_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!(" {}", v)).style(style)
                })
                .collect();

            let list = List::new(items).block(block);
            f.render_widget(list, area);
        } else {
            let paragraph = Paragraph::new("未找到服务模板").block(block);
            f.render_widget(paragraph, area);
        }
    }

    fn draw_mode_selection(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 步骤 3/5: 选择安装模式 ")
            .borders(Borders::ALL);

        let modes = vec![
            ("Systemd", "使用系统包管理器安装，由 systemd 管理"),
            ("Panel1", "由 Panel1 下载和管理二进制文件"),
            ("Docker", "使用 Docker 容器运行"),
        ];

        let items: Vec<ListItem> = modes
            .iter()
            .enumerate()
            .map(|(i, (name, desc))| {
                let style = if i == self.list_index {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(format!(" {} - {}", name, desc)).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_configuration(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 步骤 4/5: 配置服务 ")
            .borders(Borders::ALL);

        let modes = vec!["Systemd", "Panel1", "Docker"];
        let mode_name = modes.get(self.selected_mode).unwrap_or(&"Unknown");

        let text = format!(
            r#" 服务: {}
 版本: {}
 模式: {}

 端口号: {}

 (输入数字修改端口号，按 Enter 继续)"#,
            self.selected_service.as_deref().unwrap_or("unknown"),
            self.selected_version.as_deref().unwrap_or("latest"),
            mode_name,
            if self.config_port.is_empty() { "_" } else { &self.config_port }
        );

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_confirmation(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 步骤 5/5: 确认安装 ")
            .borders(Borders::ALL);

        let modes = vec!["Systemd", "Panel1", "Docker"];
        let mode_name = modes.get(self.selected_mode).unwrap_or(&"Unknown");

        let text = format!(
            r#" 确认以下安装配置:

 服务: {}
 版本: {}
 模式: {}
 端口: {}

 按 Enter 开始安装
 按 Esc 返回修改"#,
            self.selected_service.as_deref().unwrap_or("unknown"),
            self.selected_version.as_deref().unwrap_or("latest"),
            mode_name,
            self.config_port
        );

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_installing(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 安装中... ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let paragraph = Paragraph::new(self.install_message.clone()).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_done(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" 安装完成 ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let text = format!(
            "{} 已成功安装!\n\n按 Enter 返回主菜单。",
            self.selected_service.as_deref().unwrap_or("服务")
        );

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }
}

impl Default for InstallWizard {
    fn default() -> Self {
        Self::new()
    }
}
