//! 状态栏组件
//!
//! 提供动态状态栏，显示当前模式、快捷键提示、系统资源等信息

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::theme::Theme;

/// 状态栏组件
pub struct StatusBar {
    /// 当前模式名称
    mode: String,
    /// 快捷键提示
    hints: Vec<(&'static str, &'static str)>,
    /// 额外信息
    extra_info: Vec<String>,
}

impl StatusBar {
    /// 创建新的状态栏
    pub fn new() -> Self {
        Self {
            mode: String::new(),
            hints: Vec::new(),
            extra_info: Vec::new(),
        }
    }

    /// 设置当前模式
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = mode.into();
        self
    }

    /// 添加快捷键提示
    pub fn hint(mut self, key: &'static str, action: &'static str) -> Self {
        self.hints.push((key, action));
        self
    }

    /// 设置额外信息
    pub fn extra(mut self, info: Vec<String>) -> Self {
        self.extra_info = info;
        self
    }

    /// 绘制状态栏
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        // 构建状态栏内容
        let mut spans = Vec::new();

        // 模式名称
        if !self.mode.is_empty() {
            spans.push(Span::styled(
                format!(" {} ", self.mode),
                Theme::accent().add_modifier(ratatui::style::Modifier::BOLD),
            ));
            spans.push(Span::raw("│"));
        }

        // 快捷键提示
        for (i, (key, action)) in self.hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(format!("[{}]", key), Theme::accent()));
            spans.push(Span::styled(format!("{} ", action), Theme::text()));
        }

        // 额外信息
        if !self.extra_info.is_empty() {
            spans.push(Span::raw("│"));
            for info in &self.extra_info {
                spans.push(Span::styled(format!(" {} ", info), Theme::subtext()));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Theme::status_bar());

        f.render_widget(paragraph, area);
    }

    /// 绘制带分隔的状态栏
    pub fn draw_split(&self, f: &mut Frame, area: Rect, right_info: Option<&str>) {
        let constraints = if right_info.is_some() {
            vec![Constraint::Min(0), Constraint::Length(30)]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // 左侧内容
        self.draw(f, chunks[0]);

        // 右侧信息
        if let (Some(info), Some(right_area)) = (right_info, chunks.get(1)) {
            let paragraph = Paragraph::new(info)
                .style(Theme::subtext())
                .alignment(ratatui::layout::Alignment::Right);

            f.render_widget(paragraph, *right_area);
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态栏构建器
pub fn status_bar() -> StatusBar {
    StatusBar::new()
}
