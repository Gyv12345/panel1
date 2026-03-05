//! 软件包注册表 - 从远程源获取软件包配置
//!
//! 提供软件包索引、配置获取和二进制下载功能

mod client;
mod download;

pub use client::{PackageRegistry, RegistryConfig};
pub use download::{DownloadManager, DownloadProgress};

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// 默认的注册表 URL
pub const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/panel1/panel1-packages/main";

/// 索引缓存时间（1 小时）
pub const INDEX_CACHE_TTL: Duration = Duration::from_secs(3600);

/// 软件包索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIndex {
    /// 索引版本
    pub version: String,
    /// 更新时间
    pub updated_at: String,
    /// 软件包列表
    pub packages: Vec<PackageSummary>,
}

/// 软件包摘要（索引中的条目）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    /// 软件包 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 分类
    pub category: PackageCategory,
    /// 配置文件 URL（相对路径）
    pub config_url: String,
    /// 最新版本
    pub latest_version: String,
    /// 图标（可选）
    #[serde(default)]
    pub icon: Option<String>,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 软件包分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PackageCategory {
    /// 运行时环境
    Runtime,
    /// 数据库
    Database,
    /// Web 服务器
    WebServer,
    /// 缓存
    Cache,
    /// 消息队列
    MessageQueue,
    /// 工具
    Tool,
    /// 其他
    Other,
}

/// 完整的软件包配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// 软件包 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 分类
    pub category: PackageCategory,
    /// 主页 URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// 文档 URL
    #[serde(default)]
    pub documentation: Option<String>,
    /// 版本列表
    pub versions: Vec<PackageVersion>,
    /// 安装配置
    pub install: InstallConfig,
    /// 服务配置
    #[serde(default)]
    pub service: Option<ServiceConfig>,
}

/// 软件包版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    /// 版本号
    pub version: String,
    /// 是否为 LTS 版本
    #[serde(default)]
    pub lts: bool,
    /// 是否为稳定版本
    #[serde(default = "default_true")]
    pub stable: bool,
    /// 发布日期
    #[serde(default)]
    pub release_date: Option<String>,
    /// 二进制文件列表
    pub artifacts: Vec<Artifact>,
    /// 变更日志
    #[serde(default)]
    pub changelog: Option<String>,
}
/// 执行 `default_true`。
fn default_true() -> bool {
    true
}

/// 二进制文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// 目标操作系统
    pub os: String,
    /// 目标架构
    pub arch: String,
    /// 下载 URL
    pub url: String,
    /// 校验和
    pub checksum: Checksum,
    /// 文件大小（字节）
    #[serde(default)]
    pub size: Option<u64>,
    /// 压缩格式
    #[serde(default = "default_archive_format")]
    pub archive_format: String,
}
/// 执行 `default_archive_format`。
fn default_archive_format() -> String {
    "tar.gz".to_string()
}

/// 校验和信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// 算法（sha256, sha512, md5）
    pub algorithm: String,
    /// 校验和值
    pub value: String,
}

/// 安装配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// 二进制配置
    #[serde(default)]
    pub binary: Option<BinaryConfig>,
    /// 安装前执行的命令
    #[serde(default)]
    pub pre_install: Option<Vec<String>>,
    /// 安装后执行的命令
    #[serde(default)]
    pub post_install: Option<Vec<String>>,
    /// 需要创建的目录
    #[serde(default)]
    pub directories: Vec<String>,
    /// 需要的环境变量
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
}

/// 二进制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    /// 二进制文件在压缩包中的路径
    pub path: String,
    /// 需要创建的符号链接
    #[serde(default)]
    pub symlinks: Vec<String>,
    /// 是否需要设置可执行权限
    #[serde(default = "default_true")]
    pub executable: bool,
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// 默认端口
    #[serde(default)]
    pub default_port: Option<u16>,
    /// 启动命令模板
    pub start_command: String,
    /// 停止命令模板
    #[serde(default)]
    pub stop_command: Option<String>,
    /// 健康检查命令
    #[serde(default)]
    pub health_check: Option<String>,
    /// 配置文件路径
    #[serde(default)]
    pub config_file: Option<String>,
    /// 数据目录
    #[serde(default)]
    pub data_dir: Option<String>,
    /// 日志目录
    #[serde(default)]
    pub log_dir: Option<String>,
}

/// 缓存的索引
#[derive(Debug, Clone)]
pub struct CachedIndex {
    /// 索引数据
    pub index: PackageIndex,
    /// 缓存时间
    pub cached_at: SystemTime,
}

impl CachedIndex {
    /// 创建新的缓存索引
    pub fn new(index: PackageIndex) -> Self {
        Self {
            index,
            cached_at: SystemTime::now(),
        }
    }

    /// 检查缓存是否过期
    pub fn is_expired(&self) -> bool {
        self.cached_at
            .elapsed()
            .map(|d| d > INDEX_CACHE_TTL)
            .unwrap_or(true)
    }
}

/// 获取当前系统信息
pub fn current_system_info() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        "unknown"
    };

    (os, arch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：验证 package category serde。
    #[test]
    fn test_package_category_serde() {
        let json = r#""runtime""#;
        let category: PackageCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category, PackageCategory::Runtime);

        let json = r#""database""#;
        let category: PackageCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category, PackageCategory::Database);
    }

    /// 测试：验证 package summary serde。
    #[test]
    fn test_package_summary_serde() {
        let json = r#"{
            "id": "nodejs",
            "name": "Node.js",
            "description": "JavaScript 运行时环境",
            "category": "runtime",
            "config_url": "packages/nodejs.json",
            "latest_version": "20.11.0"
        }"#;

        let summary: PackageSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.id, "nodejs");
        assert_eq!(summary.name, "Node.js");
        assert_eq!(summary.category, PackageCategory::Runtime);
    }

    /// 测试：验证 cached index expiration。
    #[test]
    fn test_cached_index_expiration() {
        let index = PackageIndex {
            version: "1.0.0".to_string(),
            updated_at: "2024-01-15T10:30:00Z".to_string(),
            packages: vec![],
        };

        let cached = CachedIndex::new(index);
        // 刚创建的缓存不应该过期
        assert!(!cached.is_expired());
    }
}
