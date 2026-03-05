//! 进程管理模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{Pid, System};

/// 进程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// 进程 ID
    pub pid: u32,
    /// 父进程 ID
    pub ppid: u32,
    /// 进程名称
    pub name: String,
    /// 命令行
    pub command: String,
    /// 执行路径
    pub exe: String,
    /// 当前工作目录
    pub cwd: String,
    /// 进程状态
    pub status: String,
    /// CPU 使用率
    pub cpu_usage: f32,
    /// 内存使用 (字节)
    pub memory: u64,
    /// 虚拟内存 (字节)
    pub virtual_memory: u64,
    /// 用户
    pub user: String,
    /// 启动时间
    pub start_time: u64,
    /// 运行时间 (秒)
    pub run_time: u64,
}

/// 进程管理器
pub struct ProcessManager {
    sys: System,
    boot_time: u64,
}

impl ProcessManager {
    /// 创建新的进程管理器
    pub fn new() -> Self {
        let sys = System::new_all();
        // Get boot time from /proc/stat on Linux
        let boot_time = std::fs::read_to_string("/proc/stat")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("btime "))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or(0);

        Self { sys, boot_time }
    }

    /// 刷新进程信息
    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    /// 获取所有进程列表
    pub fn get_processes(&mut self) -> Vec<ProcessInfo> {
        use sysinfo::ProcessesToUpdate;
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );

        let boot_time = self.boot_time;

        self.sys
            .processes()
            .iter()
            .map(|(pid, process)| self.process_to_info(*pid, process, boot_time))
            .collect()
    }

    /// 根据 PID 获取进程信息
    pub fn get_process(&mut self, pid: u32) -> Option<ProcessInfo> {
        use sysinfo::ProcessesToUpdate;
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );

        let boot_time = self.boot_time;

        self.sys
            .process(Pid::from_u32(pid))
            .map(|p| self.process_to_info(Pid::from_u32(pid), p, boot_time))
    }

    /// 按名称搜索进程
    pub fn find_by_name(&mut self, name: &str) -> Vec<ProcessInfo> {
        use sysinfo::ProcessesToUpdate;
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );

        let boot_time = self.boot_time;

        self.sys
            .processes()
            .iter()
            .filter(|(_, p)| p.name().to_string_lossy().contains(name))
            .map(|(pid, process)| self.process_to_info(*pid, process, boot_time))
            .collect()
    }

    /// 终止进程
    pub fn kill(&self, pid: u32) -> bool {
        if let Some(process) = self.sys.process(Pid::from_u32(pid)) {
            process.kill()
        } else {
            false
        }
    }

    /// 获取进程树
    pub fn get_process_tree(&mut self) -> HashMap<u32, Vec<u32>> {
        use sysinfo::ProcessesToUpdate;
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );

        let mut tree: HashMap<u32, Vec<u32>> = HashMap::new();

        for (pid, process) in self.sys.processes() {
            let ppid = process.parent().map(|p| p.as_u32()).unwrap_or(0);
            tree.entry(ppid).or_default().push(pid.as_u32());
        }

        tree
    }

    /// 转换进程信息
    fn process_to_info(&self, pid: Pid, process: &sysinfo::Process, boot_time: u64) -> ProcessInfo {
        ProcessInfo {
            pid: pid.as_u32(),
            ppid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
            name: process.name().to_string_lossy().to_string(),
            command: process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" "),
            exe: process
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            cwd: process
                .cwd()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            status: format!("{:?}", process.status()),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
            virtual_memory: process.virtual_memory(),
            user: process
                .user_id()
                .map(|uid| format!("{:?}", uid))
                .unwrap_or_else(|| "unknown".to_string()),
            start_time: process.start_time(),
            run_time: boot_time.saturating_sub(process.start_time()),
        }
    }
}

impl Default for ProcessManager {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}
