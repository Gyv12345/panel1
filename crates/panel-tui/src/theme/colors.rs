//! Catppuccin Mocha 配色方案
//!
//! 现代暗色主题，提供舒适的视觉体验

use ratatui::style::Color;

/// Catppuccin Mocha 调色板
///
/// 基于 https://github.com/catppuccin/catppuccin
pub struct CatppuccinMocha;

impl CatppuccinMocha {
    // === 背景色 ===

    /// 主背景色 - #1e1e2e
    pub const BASE: Color = Color::Rgb(30, 30, 46);

    /// 次背景色 - #181825
    pub const MANTLE: Color = Color::Rgb(24, 24, 37);

    /// 卡片背景色 - #313244
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);

    /// 高亮背景色 - #45475a
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);

    /// 更高亮背景色 - #585b70
    pub const SURFACE2: Color = Color::Rgb(88, 91, 112);

    // === 文本色 ===

    /// 主文本色 - #cdd6f4
    pub const TEXT: Color = Color::Rgb(205, 214, 244);

    /// 次要文本色 - #bac2de
    pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);

    /// 更次要文本色 - #a6adc8
    pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);

    // === 强调色 ===

    /// 主强调色（紫色）- #cba6f7
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);

    /// 次强调色（蓝色）- #89b4fa
    pub const BLUE: Color = Color::Rgb(137, 180, 250);

    /// 天蓝色 - #89dceb
    pub const SKY: Color = Color::Rgb(137, 220, 235);

    /// 青色 - #94e2d5
    pub const TEAL: Color = Color::Rgb(148, 226, 213);

    /// 薰衣草色 - #b4befe
    pub const LAVENDER: Color = Color::Rgb(180, 190, 254);

    // === 状态色 ===

    /// 成功/绿色 - #a6e3a1
    pub const GREEN: Color = Color::Rgb(166, 227, 161);

    /// 警告/黄色 - #f9e2af
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);

    /// 错误/红色 - #f38ba8
    pub const RED: Color = Color::Rgb(243, 139, 168);

    /// 信息/蓝绿色 - #94e2d5
    pub const TEAL_STATUS: Color = Color::Rgb(148, 226, 213);

    // === 额外颜色 ===

    /// 粉色 - #f5c2e7
    pub const PINK: Color = Color::Rgb(245, 194, 231);

    /// 桃色 - #fab387
    pub const PEACH: Color = Color::Rgb(250, 179, 135);

    /// 橙色 - #cba6f7 (使用 Mauve)
    pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);

    /// 玫瑰色 - #f2cdcd
    pub const ROSEWATER: Color = Color::Rgb(242, 205, 205);

    /// 根据使用率返回对应颜色
    ///
    /// - < 50%: 绿色
    /// - 50% - 80%: 黄色
    /// - > 80%: 红色
    pub fn usage_color(usage: f32) -> Color {
        if usage < 50.0 {
            Self::GREEN
        } else if usage < 80.0 {
            Self::YELLOW
        } else {
            Self::RED
        }
    }

    /// 根据服务状态返回对应颜色
    pub fn service_status_color(status: ServiceStatusColor) -> Color {
        match status {
            ServiceStatusColor::Running => Self::GREEN,
            ServiceStatusColor::Stopped => Self::SUBTEXT0,
            ServiceStatusColor::Failed => Self::RED,
            ServiceStatusColor::Loading => Self::YELLOW,
            ServiceStatusColor::Unknown => Self::SUBTEXT1,
        }
    }
}

/// 服务状态颜色枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceStatusColor {
    Running,
    Stopped,
    Failed,
    Loading,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：验证 usage color。
    #[test]
    fn test_usage_color() {
        assert_eq!(CatppuccinMocha::usage_color(30.0), CatppuccinMocha::GREEN);
        assert_eq!(CatppuccinMocha::usage_color(60.0), CatppuccinMocha::YELLOW);
        assert_eq!(CatppuccinMocha::usage_color(90.0), CatppuccinMocha::RED);
    }
}
