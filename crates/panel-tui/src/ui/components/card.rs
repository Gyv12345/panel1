//! 卡片组件
//!
//! 提供带边框和标题的卡片容器，用于分组显示相关信息

use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

use crate::theme::Theme;

/// 卡片组件
pub struct Card<'a> {
    /// 标题
    title: Option<String>,
    /// 内容
    content: String,
    /// 样式
    style: CardStyle,
    /// 内边距
    padding: Padding,
    /// 标题图标
    icon: Option<&'a str>,
}

/// 卡片样式
#[derive(Debug, Clone, Copy)]
pub enum CardStyle {
    /// 默认样式
    Default,
    /// 信息样式
    Info,
    /// 成功样式
    Success,
    /// 警告样式
    Warning,
    /// 错误样式
    Error,
}

impl<'a> Card<'a> {
    /// 创建新卡片
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            title: None,
            content: content.into(),
            style: CardStyle::Default,
            padding: Padding::uniform(1),
            icon: None,
        }
    }

    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置样式
    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置图标
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 设置内边距
    pub fn padding(mut self, left: u16, right: u16, top: u16, bottom: u16) -> Self {
        self.padding = Padding::new(left, right, top, bottom);
        self
    }

    /// 获取边框样式
    fn border_style(&self) -> Style {
        match self.style {
            CardStyle::Default => Theme::border(),
            CardStyle::Info => Theme::border_accent(),
            CardStyle::Success => Theme::border_success(),
            CardStyle::Warning => Theme::warning(),
            CardStyle::Error => Theme::border_error(),
        }
    }

    /// 绘制卡片
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .padding(self.padding);

        // 设置标题
        if let Some(ref title) = self.title {
            let title_text = if let Some(icon) = self.icon {
                format!(" {} {} ", icon, title)
            } else {
                format!(" {} ", title)
            };
            block = block.title(title_text);
        }

        let paragraph = Paragraph::new(self.content.clone())
            .block(block)
            .style(Theme::text());

        f.render_widget(paragraph, area);
    }
}

/// 卡片构建器
pub fn card(content: impl Into<String>) -> Card<'static> {
    Card::new(content)
}

/// 系统信息卡片
pub struct InfoCard<'a> {
    /// 卡片基础
    base: Card<'a>,
    /// 信息项列表
    items: Vec<(String, String)>,
}

impl<'a> InfoCard<'a> {
    /// 创建新信息卡片
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            base: Card::new(String::new()).title(title).icon("◈"),
            items: Vec::new(),
        }
    }

    /// 添加信息项
    pub fn item(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.items.push((label.into(), value.into()));
        self
    }

    /// 绘制信息卡片
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let content: String = self
            .items
            .iter()
            .map(|(label, value)| format!("{}  {}", label, value))
            .collect::<Vec<_>>()
            .join("\n");

        let card = Card::new(content)
            .title(self.base.title.clone().unwrap_or_default())
            .style(self.base.style);

        let mut final_card = card;
        if let Some(icon) = self.base.icon {
            final_card = final_card.icon(icon);
        }

        final_card.draw(f, area);
    }
}

/// 信息卡片构建器
pub fn info_card(title: impl Into<String>) -> InfoCard<'static> {
    InfoCard::new(title)
}
