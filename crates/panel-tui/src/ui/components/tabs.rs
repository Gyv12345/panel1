//! 标签导航栏组件
//!
//! 提供现代化的标签式导航，支持数字快捷键

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::theme::Theme;

/// 标签页定义
#[derive(Debug, Clone)]
pub struct Tab {
    /// 标签名称
    pub name: &'static str,
    /// 快捷键
    pub key: char,
}

/// 预定义的标签页
pub const APP_TABS: &[Tab] = &[
    Tab {
        name: "仪表盘",
        key: '1',
    },
    Tab {
        name: "服务",
        key: '2',
    },
    Tab {
        name: "安装",
        key: '3',
    },
    Tab {
        name: "AI",
        key: '4',
    },
    Tab {
        name: "设置",
        key: '5',
    },
];

/// 标签导航栏
pub struct TabBar {
    /// 标签页列表
    tabs: &'static [Tab],
    /// 当前选中的索引
    selected: usize,
}

impl TabBar {
    /// 创建新的标签导航栏
    pub fn new() -> Self {
        Self {
            tabs: APP_TABS,
            selected: 0,
        }
    }

    /// 设置选中索引
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.tabs.len().saturating_sub(1));
    }

    /// 获取选中索引
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// 根据快捷键获取标签索引
    pub fn index_from_key(&self, key: char) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.key == key)
    }

    /// 绘制标签导航栏
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let titles: Vec<String> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                if i == self.selected {
                    format!(" {} ", tab.name)
                } else {
                    format!("[{}] {} ", tab.key, tab.name)
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .style(Theme::header_bar()),
            )
            .select(self.selected)
            .style(Theme::tab_inactive())
            .highlight_style(Theme::tab_active());

        f.render_widget(tabs, area);
    }

    /// 绘制带版本信息的标签导航栏
    pub fn draw_with_version(&self, f: &mut Frame, area: Rect, version: &str) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::Paragraph,
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(15)])
            .split(area);

        // 绘制标签
        self.draw(f, chunks[0]);

        // 绘制版本
        let version_text = Paragraph::new(format!(" Panel1 {} ", version))
            .style(Theme::subtext())
            .alignment(ratatui::layout::Alignment::Right);

        f.render_widget(version_text, chunks[1]);
    }
}

impl Default for TabBar {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}
