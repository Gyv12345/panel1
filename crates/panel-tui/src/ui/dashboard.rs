//! 系统仪表盘组件

use crossterm::event::{KeyCode, KeyEvent};
use panel_core::SystemMonitor;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};
use std::cell::RefCell;

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

        // 创建布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // 系统信息
                Constraint::Length(4),  // CPU
                Constraint::Length(4),  // 内存
                Constraint::Min(0),     // 磁盘
            ])
            .split(area);

        // 系统信息
        self.draw_system_info(f, chunks[0], &sys_info);

        // CPU 使用率
        self.draw_cpu_gauge(f, chunks[1], &cpu_info);

        // 内存使用率
        self.draw_memory_gauge(f, chunks[2], &mem_info);

        // 磁盘信息
        self.draw_disk_table(f, chunks[3], &disk_info);
    }

    fn draw_system_info(&self, f: &mut Frame, area: Rect, info: &panel_core::SystemInfo) {
        let block = Block::default()
            .title(" 系统信息 ")
            .borders(Borders::ALL);

        let uptime_hours = info.uptime / 3600;
        let uptime_mins = (info.uptime % 3600) / 60;

        let text = format!(
            r#" 主机名: {}
 操作系统: {} {}
 内核版本: {}
 架构: {}
 运行时间: {} 小时 {} 分钟"#,
            info.hostname,
            info.os_name,
            info.os_version,
            info.kernel_version,
            info.arch,
            uptime_hours,
            uptime_mins
        );

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_cpu_gauge(&self, f: &mut Frame, area: Rect, info: &panel_core::CpuInfo) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // CPU 信息
        let info_block = Block::default()
            .title(" CPU ")
            .borders(Borders::ALL);
        let info_text = format!(" {} ({} 核心)", info.brand, info.cores);
        let info_paragraph = Paragraph::new(info_text).block(info_block);
        f.render_widget(info_paragraph, chunks[0]);

        // 使用率
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" 使用率 "))
            .gauge_style(Style::default().fg(self.get_usage_color(info.usage)))
            .percent(info.usage as u16);
        f.render_widget(gauge, chunks[1]);
    }

    fn draw_memory_gauge(&self, f: &mut Frame, area: Rect, info: &panel_core::MemoryInfo) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // 内存信息
        let info_block = Block::default()
            .title(" 内存 ")
            .borders(Borders::ALL);
        let used_gb = info.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = info.total as f64 / 1024.0 / 1024.0 / 1024.0;
        let info_text = format!(" {:.1} GB / {:.1} GB", used_gb, total_gb);
        let info_paragraph = Paragraph::new(info_text).block(info_block);
        f.render_widget(info_paragraph, chunks[0]);

        // 使用率
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" 使用率 "))
            .gauge_style(Style::default().fg(self.get_usage_color(info.usage)))
            .percent(info.usage as u16);
        f.render_widget(gauge, chunks[1]);
    }

    fn draw_disk_table(&self, f: &mut Frame, area: Rect, disks: &[panel_core::DiskInfo]) {
        let block = Block::default()
            .title(" 磁盘 ")
            .borders(Borders::ALL);

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
                .style(Style::default().fg(self.get_usage_color(d.usage)))
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
        .header(
            Row::new(vec!["挂载点", "类型", "总量", "已用", "使用率"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(block);

        f.render_widget(table, area);
    }

    fn get_usage_color(&self, usage: f32) -> Color {
        if usage < 50.0 {
            Color::Green
        } else if usage < 80.0 {
            Color::Yellow
        } else {
            Color::Red
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}
