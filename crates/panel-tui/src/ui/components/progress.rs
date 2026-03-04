//! 进度条组件
//!
//! 提供彩色进度条，根据数值自动选择颜色

use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Gauge},
    Frame,
};

use crate::theme::{CatppuccinMocha, Theme};

/// 进度条组件
pub struct ProgressBar<'a> {
    /// 标签
    label: Option<String>,
    /// 当前值
    value: u16,
    /// 最大值
    max: u16,
    /// 是否显示百分比
    show_percent: bool,
    /// 自定义颜色
    color: Option<Style>,
    /// 标题
    title: Option<&'a str>,
}

impl<'a> ProgressBar<'a> {
    /// 创建新进度条
    pub fn new() -> Self {
        Self {
            label: None,
            value: 0,
            max: 100,
            show_percent: true,
            color: None,
            title: None,
        }
    }

    /// 设置当前值
    pub fn value(mut self, value: u16) -> Self {
        self.value = value.min(self.max);
        self
    }

    /// 设置最大值
    pub fn max(mut self, max: u16) -> Self {
        self.max = max;
        self
    }

    /// 设置标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置标题
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// 设置自定义颜色
    pub fn color(mut self, color: Style) -> Self {
        self.color = Some(color);
        self
    }

    /// 是否显示百分比
    pub fn show_percent(mut self, show: bool) -> Self {
        self.show_percent = show;
        self
    }

    /// 获取进度条样式（根据值自动选择颜色）
    fn gauge_style(&self) -> Style {
        if let Some(color) = self.color {
            return color;
        }

        let percent = if self.max > 0 {
            (self.value as f32 / self.max as f32 * 100.0) as u16
        } else {
            0
        };

        let color = CatppuccinMocha::usage_color(percent as f32);
        Style::default().fg(color)
    }

    /// 绘制进度条
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let percent = if self.max > 0 {
            (self.value as f32 / self.max as f32 * 100.0) as u16
        } else {
            0
        };

        let label = if let Some(ref l) = self.label {
            l.clone()
        } else if self.show_percent {
            format!("{}%", percent)
        } else {
            String::new()
        };

        let mut gauge = Gauge::default()
            .gauge_style(self.gauge_style())
            .percent(percent)
            .label(label);

        if let Some(title) = self.title {
            gauge = gauge.block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(Theme::border()),
            );
        }

        f.render_widget(gauge, area);
    }
}

impl Default for ProgressBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// 进度条构建器
pub fn progress() -> ProgressBar<'static> {
    ProgressBar::new()
}

/// 带标签的进度条（左侧标签 + 右侧进度条）
pub struct LabeledProgress<'a> {
    /// 左侧标签
    label: String,
    /// 进度条
    progress: ProgressBar<'a>,
}

impl<'a> LabeledProgress<'a> {
    /// 创建新的带标签进度条
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            progress: ProgressBar::new(),
        }
    }

    /// 设置当前值
    pub fn value(mut self, value: u16) -> Self {
        self.progress = self.progress.value(value);
        self
    }

    /// 设置最大值
    pub fn max(mut self, max: u16) -> Self {
        self.progress = self.progress.max(max);
        self
    }

    /// 绘制带标签的进度条
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::Paragraph,
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(0)])
            .split(area);

        // 标签
        let label = Paragraph::new(format!("{}:", self.label))
            .style(Theme::text())
            .alignment(ratatui::layout::Alignment::Right);

        f.render_widget(label, chunks[0]);

        // 进度条
        self.progress.draw(f, chunks[1]);
    }
}

/// 带标签的进度条构建器
pub fn labeled_progress(label: impl Into<String>) -> LabeledProgress<'static> {
    LabeledProgress::new(label)
}

/// 资源使用条（用于 CPU、内存等）
pub fn resource_usage(name: &str, usage: f32, area: Rect, f: &mut Frame) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        widgets::Paragraph,
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    // 名称
    let name_widget = Paragraph::new(format!("{}:", name))
        .style(Theme::text())
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(name_widget, chunks[0]);

    // 进度条
    progress()
        .value(usage as u16)
        .label(format!("{:.0}%", usage))
        .draw(f, chunks[1]);
}
