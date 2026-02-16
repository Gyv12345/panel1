//! 主题系统
//!
//! 提供统一的样式定义和应用

use ratatui::style::{Modifier, Style};

use super::colors::CatppuccinMocha;

/// 应用主题
///
/// 封装所有样式定义，提供统一的样式访问接口
pub struct Theme;

impl Theme {
    /// 获取调色板
    pub fn palette() -> &'static CatppuccinMocha {
        static PALETTE: CatppuccinMocha = CatppuccinMocha;
        &PALETTE
    }

    // === 基础样式 ===

    /// 默认文本样式
    pub fn text() -> Style {
        Style::default().fg(CatppuccinMocha::TEXT)
    }

    /// 次要文本样式
    pub fn subtext() -> Style {
        Style::default().fg(CatppuccinMocha::SUBTEXT1)
    }

    /// 标题样式
    pub fn title() -> Style {
        Style::default()
            .fg(CatppuccinMocha::MAUVE)
            .add_modifier(Modifier::BOLD)
    }

    /// 强调文本样式
    pub fn accent() -> Style {
        Style::default().fg(CatppuccinMocha::BLUE)
    }

    // === 交互元素样式 ===

    /// 选中文本样式
    pub fn selected() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::SURFACE1)
    }

    /// 选中文本样式（带高亮）
    pub fn selected_highlight() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::SURFACE2)
            .add_modifier(Modifier::BOLD)
    }

    /// 悬停样式
    pub fn hover() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::SURFACE0)
    }

    // === 状态样式 ===

    /// 成功样式
    pub fn success() -> Style {
        Style::default().fg(CatppuccinMocha::GREEN)
    }

    /// 警告样式
    pub fn warning() -> Style {
        Style::default().fg(CatppuccinMocha::YELLOW)
    }

    /// 错误样式
    pub fn error() -> Style {
        Style::default().fg(CatppuccinMocha::RED)
    }

    /// 信息样式
    pub fn info() -> Style {
        Style::default().fg(CatppuccinMocha::BLUE)
    }

    // === 组件样式 ===

    /// 标签页激活样式
    pub fn tab_active() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::MAUVE)
            .add_modifier(Modifier::BOLD)
    }

    /// 标签页未激活样式
    pub fn tab_inactive() -> Style {
        Style::default()
            .fg(CatppuccinMocha::SUBTEXT1)
            .bg(CatppuccinMocha::SURFACE0)
    }

    /// 卡片标题样式
    pub fn card_title() -> Style {
        Style::default()
            .fg(CatppuccinMocha::BLUE)
            .add_modifier(Modifier::BOLD)
    }

    /// 状态栏样式
    pub fn status_bar() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::MANTLE)
    }

    /// 标题栏样式
    pub fn header_bar() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::MANTLE)
    }

    // === 进度条样式 ===

    /// 进度条样式（根据百分比自动选择颜色）
    pub fn progress_bar(percent: u16) -> Style {
        Style::default().fg(CatppuccinMocha::usage_color(percent as f32))
    }

    /// 进度条背景样式
    pub fn progress_bar_bg() -> Style {
        Style::default().bg(CatppuccinMocha::SURFACE0)
    }

    // === 输入框样式 ===

    /// 输入框默认样式
    pub fn input() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::SURFACE0)
    }

    /// 输入框聚焦样式
    pub fn input_focused() -> Style {
        Style::default()
            .fg(CatppuccinMocha::TEXT)
            .bg(CatppuccinMocha::SURFACE1)
            .add_modifier(Modifier::BOLD)
    }

    // === 边框样式 ===

    /// 默认边框样式
    pub fn border() -> Style {
        Style::default().fg(CatppuccinMocha::SURFACE1)
    }

    /// 强调边框样式
    pub fn border_accent() -> Style {
        Style::default().fg(CatppuccinMocha::MAUVE)
    }

    /// 成功边框样式
    pub fn border_success() -> Style {
        Style::default().fg(CatppuccinMocha::GREEN)
    }

    /// 错误边框样式
    pub fn border_error() -> Style {
        Style::default().fg(CatppuccinMocha::RED)
    }

    /// 警告边框样式
    pub fn border_warning() -> Style {
        Style::default().fg(CatppuccinMocha::YELLOW)
    }
}

/// 便捷访问颜色
pub use super::colors::CatppuccinMocha as Colors;
