//! 网络信息模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// 接口名称
    pub name: String,
    /// MAC 地址
    pub mac: String,
    /// IP 地址列表
    pub ips: Vec<String>,
    /// 接收字节数
    pub received: u64,
    /// 发送字节数
    pub transmitted: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 发送包数
    pub packets_transmitted: u64,
}

/// 网络流量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTraffic {
    /// 接口名称
    pub interface: String,
    /// 接收速率 (字节/秒)
    pub rx_rate: f64,
    /// 发送速率 (字节/秒)
    pub tx_rate: f64,
    /// 总接收字节
    pub total_rx: u64,
    /// 总发送字节
    pub total_tx: u64,
}

/// 网络管理器
pub struct NetworkManager {
    prev_stats: HashMap<String, (u64, u64, std::time::Instant)>,
}

impl NetworkManager {
    /// 创建新的网络管理器
    pub fn new() -> Self {
        Self {
            prev_stats: HashMap::new(),
        }
    }

    /// 刷新网络信息
    pub fn refresh(&mut self) {
        // Network info is obtained on demand
    }

    /// 获取所有网络接口
    pub fn get_interfaces(&self) -> Vec<NetworkInterface> {
        use sysinfo::Networks;

        let networks = Networks::new_with_refreshed_list();

        networks
            .iter()
            .map(|(name, data)| NetworkInterface {
                name: name.to_string(),
                mac: data.mac_address().to_string(),
                ips: Vec::new(),
                received: data.total_received(),
                transmitted: data.total_transmitted(),
                packets_received: data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
            })
            .collect()
    }

    /// 获取网络流量统计
    pub fn get_traffic(&mut self) -> Vec<NetworkTraffic> {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();

        let now = std::time::Instant::now();
        let mut traffic = Vec::new();

        for (name, data) in networks.iter() {
            let rx = data.total_received();
            let tx = data.total_transmitted();

            let (rx_rate, tx_rate) =
                if let Some(&(prev_rx, prev_tx, prev_time)) = self.prev_stats.get(name.as_str()) {
                    let elapsed = now.duration_since(prev_time).as_secs_f64();
                    if elapsed > 0.0 {
                        (
                            (rx.saturating_sub(prev_rx) as f64 / elapsed),
                            (tx.saturating_sub(prev_tx) as f64 / elapsed),
                        )
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };

            traffic.push(NetworkTraffic {
                interface: name.to_string(),
                rx_rate,
                tx_rate,
                total_rx: rx,
                total_tx: tx,
            });

            self.prev_stats.insert(name.to_string(), (rx, tx, now));
        }

        traffic
    }

    /// 获取端口占用情况
    pub fn get_listening_ports(&self) -> Result<Vec<ListeningPort>, anyhow::Error> {
        let output = std::process::Command::new("ss")
            .args(["-tlnp", "--no-header"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ports: Vec<ListeningPort> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let local_addr = parts[3];
                            if let Some(port_str) = local_addr.rsplit(':').next() {
                                if let Ok(port) = port_str.parse::<u16>() {
                                    let process = if parts.len() > 5 {
                                        parts[5].to_string()
                                    } else {
                                        String::new()
                                    };
                                    return Some(ListeningPort {
                                        port,
                                        protocol: "tcp".to_string(),
                                        address: local_addr.to_string(),
                                        process,
                                    });
                                }
                            }
                        }
                        None
                    })
                    .collect();
                Ok(ports)
            }
            Err(_) => {
                // ss 命令不存在，尝试 netstat
                let output = std::process::Command::new("netstat")
                    .args(["-tlnp"])
                    .output();

                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let ports: Vec<ListeningPort> = stdout
                            .lines()
                            .filter_map(|line| {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 4 {
                                    let local_addr = parts[3];
                                    if let Some(port_str) = local_addr.rsplit(':').next() {
                                        if let Ok(port) = port_str.parse::<u16>() {
                                            return Some(ListeningPort {
                                                port,
                                                protocol: "tcp".to_string(),
                                                address: local_addr.to_string(),
                                                process: parts.get(6).unwrap_or(&"").to_string(),
                                            });
                                        }
                                    }
                                }
                                None
                            })
                            .collect();
                        Ok(ports)
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to get listening ports: {}", e)),
                }
            }
        }
    }
}

impl Default for NetworkManager {
    /// 返回默认实例。
    fn default() -> Self {
        Self::new()
    }
}

/// 监听端口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningPort {
    /// 端口号
    pub port: u16,
    /// 协议
    pub protocol: String,
    /// 监听地址
    pub address: String,
    /// 占用进程
    pub process: String,
}
