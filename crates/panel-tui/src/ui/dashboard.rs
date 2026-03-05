//! 系统仪表盘组件
//!
//! 使用 Catppuccin Mocha 主题的现代化仪表盘

use crossterm::event::{KeyCode, KeyEvent};
use panel_core::SystemMonitor;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};
use std::cell::RefCell;

use crate::theme::{CatppuccinMocha, Theme};
use crate::ui::components::{info_card, resource_usage};

/// 仪表盘组件
pub struct Dashboard {
    scroll_offset: usize,
}

impl Dashboard {
    /// 创建新的仪表盘
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// 处理按键
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
            }
            KeyCode::Char('r') => {
                // 刷新由主循环处理
            }
            _ => {}
        }
    }

    /// 绘制仪表盘
    pub fn draw(&self, f: &mut Frame, area: Rect, monitor: &RefCell<SystemMonitor>) {
        let m = monitor.borrow();
        let sys_info = m.get_system_info();
        let disk_info = m.get_disk_info();
        let mem_info = m.get_memory_info();
        drop(m);

        let mut m = monitor.borrow_mut();
        let cpu_info = m.get_cpu_info();
        drop(m);

        // 创建卡片式布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9), // 系统信息 + 资源监控
                Constraint::Length(5), // CPU
                Constraint::Length(5), // 内存
                Constraint::Min(0),    // 磁盘
            ])
            .split(area);

        // 系统概览和资源监控（横向排列）
        self.draw_overview_row(f, chunks[0], &sys_info, &cpu_info, &mem_info);

        // CPU 详细信息
        self.draw_cpu_section(f, chunks[1], &cpu_info);

        // 内存详细信息
        self.draw_memory_section(f, chunks[2], &mem_info);

        // 磁盘信息
        self.draw_disk_table(f, chunks[3], &disk_info);
    }
    /// 绘制 overview row。

    fn draw_overview_row(
        &self,
        f: &mut Frame,
        area: Rect,
        info: &panel_core::SystemInfo,
        cpu: &panel_core::CpuInfo,
        mem: &panel_core::MemoryInfo,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // 系统概览
                Constraint::Percentage(35), // 资源监控
                Constraint::Percentage(25), // 快捷键
            ])
            .split(area);

        // 系统概览卡片
        let uptime_hours = info.uptime / 3600;
        let uptime_mins = (info.uptime % 3600) / 60;

        info_card("系统概览")
            .item("◈ Hostname", &info.hostname)
            .item("◈ OS", format!("{} {}", info.os_name, info.os_version))
            .item("◈ Kernel", &info.kernel_version)
            .item("◈ Arch", &info.arch)
            .item("◈ Uptime", format!("{}h {}m", uptime_hours, uptime_mins))
            .draw(f, chunks[0]);

        // 资源监控卡片
        self.draw_resource_card(f, chunks[1], cpu, mem);

        // 快捷键卡片
        self.draw_shortcuts_card(f, chunks[2]);
    }
    /// 绘制 resource card。

    fn draw_resource_card(
        &self,
        f: &mut Frame,
        area: Rect,
        cpu: &panel_core::CpuInfo,
        mem: &panel_core::MemoryInfo,
    ) {
        use ratatui::widgets::Block;

        let block = Block::default()
            .title(" 资源监控 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let inner = block.inner(area);
        f.render_widget(block, area);

        // 内部分割
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(inner);

        // CPU 使用率
        resource_usage("CPU", cpu.usage, chunks[0], f);

        // 内存使用率
        resource_usage("MEM", mem.usage, chunks[1], f);
    }
    /// 绘制 shortcuts card。

    fn draw_shortcuts_card(&self, f: &mut Frame, area: Rect) {
        use ratatui::widgets::Block;

        let block = Block::default()
            .title(" 快捷键 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let inner = block.inner(area);
        f.render_widget(block, area);

        let text = "[r] 刷新\n[?] 帮助\n[q] 退出";

        let paragraph = Paragraph::new(text)
            .style(Theme::subtext())
            .alignment(ratatui::layout::Alignment::Left);

        f.render_widget(paragraph, inner);
    }
    /// 绘制 cpu section。

    fn draw_cpu_section(&self, f: &mut Frame, area: Rect, info: &panel_core::CpuInfo) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // CPU 信息
        let info_block = Block::default()
            .title(" CPU ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let info_text = format!(" {} ({} 核心)", info.brand, info.cores);
        let info_paragraph = Paragraph::new(info_text)
            .style(Theme::text())
            .block(info_block);

        f.render_widget(info_paragraph, chunks[0]);

        // 使用率进度条
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Theme::border())
                    .title(" 使用率 "),
            )
            .gauge_style(
                ratatui::style::Style::default().fg(CatppuccinMocha::usage_color(info.usage)),
            )
            .percent(info.usage as u16)
            .label(format!("{:.1}%", info.usage));

        f.render_widget(gauge, chunks[1]);
    }
    /// 绘制 memory section。

    fn draw_memory_section(&self, f: &mut Frame, area: Rect, info: &panel_core::MemoryInfo) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // 内存信息
        let info_block = Block::default()
            .title(" 内存 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let used_gb = info.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = info.total as f64 / 1024.0 / 1024.0 / 1024.0;
        let info_text = format!(" {:.1} GB / {:.1} GB", used_gb, total_gb);
        let info_paragraph = Paragraph::new(info_text)
            .style(Theme::text())
            .block(info_block);

        f.render_widget(info_paragraph, chunks[0]);

        // 使用率进度条
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Theme::border())
                    .title(" 使用率 "),
            )
            .gauge_style(
                ratatui::style::Style::default().fg(CatppuccinMocha::usage_color(info.usage)),
            )
            .percent(info.usage as u16)
            .label(format!("{:.1}%", info.usage));

        f.render_widget(gauge, chunks[1]);
    }
    /// 绘制 disk table。

    fn draw_disk_table(&self, f: &mut Frame, area: Rect, disks: &[panel_core::DiskInfo]) {
        let block = Block::default()
            .title(" 磁盘 ")
            .title_style(Theme::card_title())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let rows: Vec<Row> = disks
            .iter()
            .map(|d| {
                let total_gb = d.total as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_gb = d.used as f64 / 1024.0 / 1024.0 / 1024.0;
                Row::new(vec![
                    d.mount_point.clone(),
                    d.fs_type.clone(),
                    format!("{:.1} GB", total_gb),
                    format!("{:.1} GB", used_gb),
                    format!("{:.1}%", d.usage),
                ])
                .style(ratatui::style::Style::default().fg(CatppuccinMocha::usage_color(d.usage)))
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
            ],
        )
        .header(Row::new(vec!["挂载点", "类型", "总量", "已用", "使用率"]).style(Theme::accent()))
        .block(block);

        f.render_widget(table, area);
    }
}

impl Default for Dashboard {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}
