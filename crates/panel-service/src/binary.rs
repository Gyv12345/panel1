//! 二进制后端 - 下载和管理 Panel1 托管的二进制服务

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::manager::{ManagedService, ServiceStatus};
use crate::registry::{
    DownloadManager, DownloadProgress, PackageConfig, PackageRegistry, PackageSummary,
    RegistryConfig,
};

/// 二进制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    /// 下载 URL 模板
    pub download_url: String,
    /// 校验和（可选）
    pub checksum: Option<String>,
    /// 默认端口
    pub default_port: u16,
    /// 启动命令
    pub start_command: String,
    /// 配置文件模板
    pub config_template: Option<String>,
}

/// URL 安装模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UrlInstallMode {
    /// 自动检测（默认）
    Auto,
    /// 二进制直装模式
    Panel1,
    /// Docker 方案模式（会自动检查并安装 Docker 环境）
    Docker,
}

impl UrlInstallMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            UrlInstallMode::Auto => "auto",
            UrlInstallMode::Panel1 => "panel1",
            UrlInstallMode::Docker => "docker",
        }
    }
}

impl Default for UrlInstallMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// 进程守护器
pub struct ProcessGuard {
    /// 进程映射表
    processes: Arc<RwLock<HashMap<String, u32>>>,
}

impl ProcessGuard {
    /// 创建新的进程守护器
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册进程
    pub async fn register(&self, name: &str, pid: u32) {
        let mut processes = self.processes.write().await;
        processes.insert(name.to_string(), pid);
        info!("Registered process {} with PID {}", name, pid);
    }

    /// 注销进程
    pub async fn unregister(&self, name: &str) {
        let mut processes = self.processes.write().await;
        processes.remove(name);
        info!("Unregistered process {}", name);
    }

    /// 获取进程 PID
    pub async fn get_pid(&self, name: &str) -> Option<u32> {
        let processes = self.processes.read().await;
        processes.get(name).copied()
    }

    /// 检查进程是否运行
    pub async fn is_running(&self, name: &str) -> bool {
        if let Some(pid) = self.get_pid(name).await {
            // 检查进程是否存在
            let path = PathBuf::from("/proc").join(pid.to_string());
            path.exists()
        } else {
            false
        }
    }
}

impl Default for ProcessGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// 二进制后端
pub struct BinaryBackend {
    /// 数据目录
    data_dir: PathBuf,
    /// 进程守护器
    process_guard: ProcessGuard,
    /// 软件包注册表
    registry: Option<PackageRegistry>,
    /// 下载管理器
    downloader: Option<DownloadManager>,
}

fn read_env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn detect_is_root_user() -> bool {
    read_env_non_empty("EUID").as_deref() == Some("0")
        || read_env_non_empty("UID").as_deref() == Some("0")
        || read_env_non_empty("USER").as_deref() == Some("root")
}

fn resolve_default_data_dir(
    panel_service_dir: Option<&str>,
    panel_data_dir: Option<&str>,
    home_dir: Option<&str>,
    is_root: bool,
    is_linux: bool,
) -> PathBuf {
    if let Some(custom_dir) = panel_service_dir {
        return PathBuf::from(custom_dir);
    }

    if let Some(panel_data_dir) = panel_data_dir {
        return PathBuf::from(panel_data_dir).join("services");
    }

    if is_linux && is_root {
        return PathBuf::from("/opt/panel/services");
    }

    if let Some(home_dir) = home_dir {
        return PathBuf::from(home_dir).join(".panel1/services");
    }

    PathBuf::from(".panel1/services")
}

fn ensure_writable_dir(path: &Path) -> bool {
    if std::fs::create_dir_all(path).is_err() {
        return false;
    }

    let probe_dir = path.join(format!(".panel1-write-test-{}", std::process::id()));
    match std::fs::create_dir(&probe_dir) {
        Ok(()) => {
            let _ = std::fs::remove_dir(&probe_dir);
            true
        }
        Err(_) => false,
    }
}

fn resolve_writable_data_dir(primary: PathBuf, fallback_base: &Path) -> PathBuf {
    if ensure_writable_dir(&primary) {
        return primary;
    }

    let fallback = fallback_base.join(".panel1/services");
    if ensure_writable_dir(&fallback) {
        return fallback;
    }

    primary
}

impl BinaryBackend {
    /// 创建新的二进制后端
    pub fn new() -> Self {
        let panel_service_dir = read_env_non_empty("PANEL_SERVICE_DIR");
        let panel_data_dir = read_env_non_empty("PANEL_DATA_DIR");
        let home_dir = read_env_non_empty("HOME");
        let data_dir = resolve_default_data_dir(
            panel_service_dir.as_deref(),
            panel_data_dir.as_deref(),
            home_dir.as_deref(),
            detect_is_root_user(),
            cfg!(target_os = "linux"),
        );
        let fallback_base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let data_dir = resolve_writable_data_dir(data_dir, &fallback_base);
        let registry = PackageRegistry::with_defaults().ok();
        let downloader = DownloadManager::with_defaults().ok();

        Self {
            data_dir,
            process_guard: ProcessGuard::new(),
            registry,
            downloader,
        }
    }

    /// 使用自定义数据目录创建
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let registry = PackageRegistry::with_defaults().ok();
        let downloader = DownloadManager::with_defaults().ok();

        Self {
            data_dir,
            process_guard: ProcessGuard::new(),
            registry,
            downloader,
        }
    }

    /// 启用注册表功能
    pub fn with_registry(mut self, config: RegistryConfig) -> Result<Self> {
        self.registry = Some(PackageRegistry::new(config)?);
        Ok(self)
    }

    /// 列出可用的软件包
    pub async fn list_available_packages(&self) -> Result<Vec<PackageSummary>> {
        let registry = self.registry.as_ref().context("Registry not initialized")?;

        let index = registry.get_index().await?;
        Ok(index.packages)
    }

    /// 搜索软件包
    pub async fn search_packages(&self, query: &str) -> Result<Vec<PackageSummary>> {
        let registry = self.registry.as_ref().context("Registry not initialized")?;

        registry.search(query).await
    }

    /// 获取软件包配置
    pub async fn get_package_config(&self, package_id: &str) -> Result<PackageConfig> {
        let registry = self.registry.as_ref().context("Registry not initialized")?;

        registry.get_package_config(package_id).await
    }

    /// 设置下载进度回调
    pub async fn set_download_progress(
        &self,
        callback: Option<Arc<dyn Fn(DownloadProgress) + Send + Sync>>,
    ) -> Result<()> {
        let downloader = self
            .downloader
            .as_ref()
            .context("Downloader not initialized")?;

        downloader.set_progress_callback(callback).await;
        Ok(())
    }

    /// 从注册表安装服务
    pub async fn install_from_registry(
        &mut self,
        name: &str,
        package_id: &str,
        version: &str,
    ) -> Result<ManagedService> {
        let registry = self.registry.as_ref().context("Registry not initialized")?;

        let downloader = self
            .downloader
            .as_ref()
            .context("Downloader not initialized")?;

        let requested_version = if version.trim().is_empty() {
            "latest"
        } else {
            version
        };

        let resolved_version = if requested_version.eq_ignore_ascii_case("latest") {
            let index = registry.get_index().await?;
            index
                .packages
                .iter()
                .find(|pkg| pkg.id == package_id)
                .map(|pkg| pkg.latest_version.clone())
                .with_context(|| format!("Package {} not found in index", package_id))?
        } else {
            requested_version.to_string()
        };

        // 获取软件包配置
        let config = registry.get_package_config(package_id).await?;

        // 查找适合当前平台的二进制
        let artifact = registry
            .find_artifact(&config, &resolved_version)?
            .with_context(|| {
                format!(
                    "No artifact found for {} version {} on current platform",
                    package_id, resolved_version
                )
            })?;

        // 创建服务目录
        let service_dir = self.data_dir.join(name);
        tokio::fs::create_dir_all(&service_dir)
            .await
            .context("Failed to create service directory")?;

        // 确定二进制文件名
        let binary_name = extract_filename(&artifact.url);
        let binary_path = service_dir.join(&binary_name);

        // 下载二进制文件
        info!(
            "Downloading {} version {} to {:?}",
            package_id, resolved_version, binary_path
        );
        downloader.download(&artifact, &binary_path).await?;

        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))
                .await
                .context("Failed to set executable permissions")?;
        }

        // 创建符号链接
        if let Some(ref binary_config) = config.install.binary {
            for symlink in &binary_config.symlinks {
                let symlink_path = service_dir.join(symlink);
                if symlink_path.exists() {
                    tokio::fs::remove_file(&symlink_path).await.ok();
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    let relative_path = format!("./{}", binary_name);
                    symlink(&relative_path, &symlink_path).context("Failed to create symlink")?;
                }
            }
        }

        // 创建配置目录和数据目录
        if let Some(ref service_config) = config.service {
            if let Some(ref data_dir) = service_config.data_dir {
                let full_data_dir = service_dir.join(data_dir);
                tokio::fs::create_dir_all(&full_data_dir)
                    .await
                    .context("Failed to create data directory")?;
            }
            if let Some(ref log_dir) = service_config.log_dir {
                let full_log_dir = service_dir.join(log_dir);
                tokio::fs::create_dir_all(&full_log_dir)
                    .await
                    .context("Failed to create log directory")?;
            }
        }

        // 获取默认端口
        let port = config.service.as_ref().and_then(|s| s.default_port);

        Ok(ManagedService {
            id: None,
            name: name.to_string(),
            service_type: package_id.to_string(),
            mode: crate::manager::ServiceMode::Panel1,
            version: resolved_version,
            binary_path: Some(binary_path.to_string_lossy().to_string()),
            config_path: None,
            port,
            status: ServiceStatus::Stopped,
            auto_start: false,
        })
    }

    /// 安装服务
    pub async fn install(
        &mut self,
        name: &str,
        service_type: &str,
        version: &str,
    ) -> Result<ManagedService> {
        self.install_from_registry(name, service_type, version)
            .await
            .with_context(|| {
                format!(
                    "Failed to install package {} (version: {}) from registry",
                    service_type, version
                )
            })
    }

    /// 从任意 URL 安装服务（带自动重试与自修复）
    pub async fn install_from_url(
        &mut self,
        preferred_name: Option<&str>,
        raw_url: &str,
        install_mode: UrlInstallMode,
    ) -> Result<ManagedService> {
        let normalized_url = normalize_url(raw_url);
        let service_name = preferred_name
            .map(sanitize_service_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| infer_service_name(&normalized_url));
        let service_dir = self.data_dir.join(&service_name);

        let mut attempts = vec![normalized_url.clone()];
        if normalized_url.starts_with("https://") {
            attempts.push(normalized_url.replacen("https://", "http://", 1));
        }

        let mut last_error = None;

        for (idx, attempt_url) in attempts.into_iter().enumerate() {
            let attempt_no = idx + 1;
            match self
                .install_from_url_once(&service_name, &attempt_url, &service_dir, install_mode)
                .await
            {
                Ok(service) => return Ok(service),
                Err(error) => {
                    last_error = Some(error);
                    self.heal_install_directory(&service_dir).await;

                    // 避免立即重试导致同一临时状态重复失败
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    info!(
                        "Install from URL attempt {} failed, trying next strategy",
                        attempt_no
                    );
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("install from URL failed")))
            .with_context(|| format!("Failed to install service from URL: {}", raw_url))
    }

    async fn install_from_url_once(
        &self,
        service_name: &str,
        url: &str,
        service_dir: &Path,
        install_mode: UrlInstallMode,
    ) -> Result<ManagedService> {
        tokio::fs::create_dir_all(service_dir)
            .await
            .context("Failed to create service directory")?;

        if install_mode == UrlInstallMode::Docker {
            ensure_runtime_dependency(RuntimeDependency::Docker)
                .context("Docker mode requested but Docker dependency setup failed")?;
        }

        let filename = extract_filename(url);
        let downloaded_file = service_dir.join(&filename);
        download_to_file(url, &downloaded_file).await?;

        let mut binary_path = downloaded_file.clone();
        if is_archive_file(&filename) {
            match extract_archive(&downloaded_file, service_dir) {
                Ok(()) => {
                    binary_path = find_executable(service_dir, service_name)
                        .or_else(|| find_first_regular_file(service_dir))
                        .context("Archive extracted but no executable file found")?;
                }
                Err(extract_error) => {
                    info!(
                        "Archive extraction failed, fallback to raw binary: {}",
                        extract_error
                    );
                    binary_path = downloaded_file.clone();
                }
            }
        }

        set_executable_permission(&binary_path).await?;

        let dependencies =
            detect_runtime_dependencies(install_mode, &downloaded_file, &binary_path, service_dir);

        for dependency in dependencies {
            ensure_runtime_dependency(dependency).with_context(|| {
                format!(
                    "Failed to auto install runtime dependency '{}' for {}",
                    dependency.display_name(),
                    service_name
                )
            })?;
        }

        Ok(ManagedService {
            id: None,
            name: service_name.to_string(),
            service_type: service_name.to_string(),
            mode: crate::manager::ServiceMode::Panel1,
            version: "url".to_string(),
            binary_path: Some(binary_path.to_string_lossy().to_string()),
            config_path: None,
            port: None,
            status: ServiceStatus::Stopped,
            auto_start: false,
        })
    }

    async fn heal_install_directory(&self, service_dir: &Path) {
        let _ = tokio::fs::create_dir_all(service_dir).await;

        if let Ok(entries) = std::fs::read_dir(service_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_temp = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "tmp" || ext == "part")
                    .unwrap_or(false);
                if is_temp {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    /// 启动服务
    pub async fn start(&self, service: &ManagedService) -> Result<()> {
        let binary_path = service
            .binary_path
            .as_ref()
            .context("Binary path not set")?;

        if !Path::new(binary_path).exists() {
            bail!("Binary not found: {}", binary_path);
        }

        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(binary_path, std::fs::Permissions::from_mode(0o755))
                .await
                .context("Failed to set executable permissions")?;
        }

        // 启动进程
        let child = Command::new(binary_path)
            .current_dir(self.data_dir.join(&service.name))
            .spawn()
            .context("Failed to start service")?;

        let pid = child.id();
        self.process_guard.register(&service.name, pid).await;

        info!("Started service {} with PID {}", service.name, pid);
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&self, service: &ManagedService) -> Result<()> {
        if let Some(pid) = self.process_guard.get_pid(&service.name).await {
            // 发送 SIGTERM
            #[cfg(unix)]
            {
                use std::process::Command as StdCommand;
                let _ = StdCommand::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
            }

            // 等待进程退出
            for _ in 0..10 {
                if !self.process_guard.is_running(&service.name).await {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // 如果还在运行，强制杀死
            if self.process_guard.is_running(&service.name).await {
                #[cfg(unix)]
                {
                    use std::process::Command as StdCommand;
                    let _ = StdCommand::new("kill")
                        .arg("-KILL")
                        .arg(pid.to_string())
                        .output();
                }
            }

            self.process_guard.unregister(&service.name).await;
            info!("Stopped service {}", service.name);
        }

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self, service: &ManagedService) -> Result<ServiceStatus> {
        if self.process_guard.is_running(&service.name).await {
            Ok(ServiceStatus::Running)
        } else {
            Ok(ServiceStatus::Stopped)
        }
    }

    /// 卸载服务
    pub async fn uninstall(&self, service: &ManagedService) -> Result<()> {
        // 先停止服务
        self.stop(service).await?;

        // 删除服务目录
        let service_dir = self.data_dir.join(&service.name);
        if service_dir.exists() {
            tokio::fs::remove_dir_all(&service_dir)
                .await
                .context("Failed to remove service directory")?;
        }

        info!("Uninstalled service {}", service.name);
        Ok(())
    }
}

impl Default for BinaryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_runtime_dependencies, detect_runtime_from_path, resolve_default_data_dir,
        resolve_writable_data_dir, BinaryBackend, RuntimeDependency, UrlInstallMode,
    };
    use std::path::{Path, PathBuf};

    fn path_eq(actual: &Path, expected: &str) -> bool {
        actual == PathBuf::from(expected)
    }

    #[test]
    fn resolve_default_data_dir_prefers_panel_service_dir_env() {
        let path = resolve_default_data_dir(
            Some("/tmp/custom-services"),
            Some("/tmp/panel-data"),
            Some("/tmp/home"),
            false,
            true,
        );
        assert!(path_eq(&path, "/tmp/custom-services"));
    }

    #[test]
    fn resolve_default_data_dir_uses_panel_data_dir_when_set() {
        let path = resolve_default_data_dir(
            None,
            Some("/tmp/panel-data"),
            Some("/tmp/home"),
            false,
            true,
        );
        assert_eq!(path, PathBuf::from("/tmp/panel-data/services"));
    }

    #[test]
    fn resolve_default_data_dir_uses_opt_panel_for_linux_root() {
        let path = resolve_default_data_dir(None, None, Some("/tmp/home"), true, true);
        assert_eq!(path, PathBuf::from("/opt/panel/services"));
    }

    #[test]
    fn resolve_default_data_dir_falls_back_to_user_home() {
        let path = resolve_default_data_dir(None, None, Some("/tmp/home"), false, true);
        assert_eq!(path, PathBuf::from("/tmp/home/.panel1/services"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_writable_data_dir_falls_back_when_primary_is_unwritable() {
        let fallback_base = std::env::temp_dir().join("panel1-test-fallback");
        let path = resolve_writable_data_dir(PathBuf::from("/dev/null/panel1"), &fallback_base);
        assert_eq!(path, fallback_base.join(".panel1/services"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_writable_data_dir_falls_back_when_primary_exists_but_not_writable() {
        use std::os::unix::fs::PermissionsExt;

        let base = std::env::temp_dir().join("panel1-test-readonly");
        let primary = base.join("primary");
        let fallback_base = base.join("fallback");

        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&primary).expect("create primary dir");
        std::fs::set_permissions(&primary, std::fs::Permissions::from_mode(0o555))
            .expect("set readonly permission");

        let path = resolve_writable_data_dir(primary.clone(), &fallback_base);
        assert_eq!(path, fallback_base.join(".panel1/services"));

        let _ = std::fs::set_permissions(&primary, std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn new_backend_uses_writable_fallback_dir() {
        let original_service_dir = std::env::var("PANEL_SERVICE_DIR").ok();
        let original_data_dir = std::env::var("PANEL_DATA_DIR").ok();

        std::env::set_var("PANEL_SERVICE_DIR", "/dev/null/panel1");
        std::env::remove_var("PANEL_DATA_DIR");

        let backend = BinaryBackend::new();
        let expected = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".panel1/services");
        assert_eq!(backend.data_dir, expected);

        if let Some(value) = original_service_dir {
            std::env::set_var("PANEL_SERVICE_DIR", value);
        } else {
            std::env::remove_var("PANEL_SERVICE_DIR");
        }

        if let Some(value) = original_data_dir {
            std::env::set_var("PANEL_DATA_DIR", value);
        } else {
            std::env::remove_var("PANEL_DATA_DIR");
        }
    }

    #[test]
    fn infer_service_name_from_archive_url() {
        assert_eq!(
            super::infer_service_name("https://example.com/Redis-7.2.0.tar.gz"),
            "redis-7-2-0"
        );
    }

    #[test]
    fn normalize_url_adds_https() {
        assert_eq!(
            super::normalize_url("downloads.example.com/tool"),
            "https://downloads.example.com/tool"
        );
    }

    #[test]
    fn detect_runtime_from_extension() {
        assert_eq!(
            detect_runtime_from_path(Path::new("/tmp/app.py")),
            Some(RuntimeDependency::Python)
        );
        assert_eq!(
            detect_runtime_from_path(Path::new("/tmp/app.mjs")),
            Some(RuntimeDependency::Node)
        );
    }

    #[test]
    fn detect_runtime_from_shebang_script() {
        let path = std::env::temp_dir().join("panel1-runtime-detect.py");
        std::fs::write(&path, "#!/usr/bin/env python3\nprint('ok')\n").expect("write test file");
        assert_eq!(
            detect_runtime_from_path(&path),
            Some(RuntimeDependency::Python)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn detect_runtime_dependencies_respects_docker_mode() {
        let base = std::env::temp_dir().join("panel1-runtime-detect-dir");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create temp dir");

        let download = base.join("tool.tar.gz");
        let executable = base.join("entry.sh");
        std::fs::write(&download, "").expect("write fake download");
        std::fs::write(&executable, "#!/bin/sh\necho ok\n").expect("write fake executable");

        let dependencies =
            detect_runtime_dependencies(UrlInstallMode::Docker, &download, &executable, &base);
        assert!(dependencies.contains(&RuntimeDependency::Docker));

        let _ = std::fs::remove_dir_all(base);
    }
}

/// 从 URL 中提取文件名
fn extract_filename(url: &str) -> String {
    let sanitized = url.split('?').next().unwrap_or(url);
    sanitized
        .rsplit('/')
        .next()
        .filter(|v| !v.is_empty())
        .unwrap_or("binary")
        .to_string()
}

fn normalize_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    format!("https://{}", trimmed)
}

fn sanitize_service_name(name: &str) -> String {
    name.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase()
}

fn infer_service_name(url: &str) -> String {
    let mut filename = extract_filename(url).to_ascii_lowercase();

    for suffix in [".tar.gz", ".tgz", ".tar", ".zip", ".gz", ".xz"] {
        if filename.ends_with(suffix) {
            filename = filename.trim_end_matches(suffix).to_string();
            break;
        }
    }

    let candidate = sanitize_service_name(&filename);
    if candidate.is_empty() {
        "tool".to_string()
    } else {
        candidate
    }
}

fn is_archive_file(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar")
        || lower.ends_with(".zip")
}

fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<()> {
    let file = archive_path.to_string_lossy().to_string();
    let dir = target_dir.to_string_lossy().to_string();
    let lower = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let output = if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Command::new("tar")
            .args(["-xzf", &file, "-C", &dir])
            .output()
            .context("Failed to run tar for extraction")?
    } else if lower.ends_with(".tar") {
        Command::new("tar")
            .args(["-xf", &file, "-C", &dir])
            .output()
            .context("Failed to run tar for extraction")?
    } else if lower.ends_with(".zip") {
        Command::new("unzip")
            .args(["-o", &file, "-d", &dir])
            .output()
            .context("Failed to run unzip for extraction")?
    } else {
        bail!("Unsupported archive format: {}", archive_path.display());
    };

    if !output.status.success() {
        bail!(
            "Archive extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

async fn download_to_file(url: &str, target_path: &Path) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .user_agent(format!("Panel1/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download URL: {}", url))?;

    if !response.status().is_success() {
        bail!("Download failed: {} (HTTP {})", url, response.status());
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read downloaded body: {}", url))?;

    tokio::fs::write(target_path, &bytes)
        .await
        .with_context(|| format!("Failed to write downloaded file: {}", target_path.display()))?;

    Ok(())
}

fn find_first_regular_file(dir: &Path) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn find_executable(dir: &Path, preferred_name: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let preferred = preferred_name.to_ascii_lowercase();
    let mut best: Option<(i32, PathBuf)> = None;

    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !path.is_file() {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            if is_archive_file(&filename) {
                continue;
            }

            let score = if filename == preferred {
                100
            } else if filename.starts_with(&preferred) {
                80
            } else if filename.contains(&preferred) {
                50
            } else {
                10
            };

            match best {
                Some((best_score, _)) if best_score >= score => {}
                _ => best = Some((score, path)),
            }
        }
    }

    best.map(|(_, path)| path)
}

async fn set_executable_permission(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .await
            .with_context(|| format!("Failed to set executable permissions: {}", path.display()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RuntimeDependency {
    Docker,
    Node,
    Python,
}

impl RuntimeDependency {
    fn display_name(&self) -> &'static str {
        match self {
            RuntimeDependency::Docker => "docker",
            RuntimeDependency::Node => "node",
            RuntimeDependency::Python => "python",
        }
    }

    fn is_available(&self) -> bool {
        match self {
            RuntimeDependency::Docker => command_exists("docker"),
            RuntimeDependency::Node => command_exists("node") || command_exists("nodejs"),
            RuntimeDependency::Python => command_exists("python3") || command_exists("python"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinuxPackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
}

impl LinuxPackageManager {
    fn detect() -> Option<Self> {
        if command_exists("apt-get") {
            return Some(Self::Apt);
        }
        if command_exists("dnf") {
            return Some(Self::Dnf);
        }
        if command_exists("yum") {
            return Some(Self::Yum);
        }
        if command_exists("pacman") {
            return Some(Self::Pacman);
        }
        if command_exists("zypper") {
            return Some(Self::Zypper);
        }
        if command_exists("apk") {
            return Some(Self::Apk);
        }
        None
    }
}

fn command_exists(command: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(command);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

fn dependency_install_command(
    dependency: RuntimeDependency,
    package_manager: LinuxPackageManager,
) -> &'static str {
    match (dependency, package_manager) {
        (RuntimeDependency::Docker, LinuxPackageManager::Apt) => {
            "apt-get update && apt-get install -y docker.io"
        }
        (RuntimeDependency::Docker, LinuxPackageManager::Dnf) => "dnf -y install docker",
        (RuntimeDependency::Docker, LinuxPackageManager::Yum) => "yum -y install docker",
        (RuntimeDependency::Docker, LinuxPackageManager::Pacman) => "pacman -Sy --noconfirm docker",
        (RuntimeDependency::Docker, LinuxPackageManager::Zypper) => {
            "zypper --non-interactive install docker"
        }
        (RuntimeDependency::Docker, LinuxPackageManager::Apk) => "apk add docker",
        (RuntimeDependency::Node, LinuxPackageManager::Apt) => {
            "apt-get update && apt-get install -y nodejs npm"
        }
        (RuntimeDependency::Node, LinuxPackageManager::Dnf) => "dnf -y install nodejs npm",
        (RuntimeDependency::Node, LinuxPackageManager::Yum) => "yum -y install nodejs npm",
        (RuntimeDependency::Node, LinuxPackageManager::Pacman) => {
            "pacman -Sy --noconfirm nodejs npm"
        }
        (RuntimeDependency::Node, LinuxPackageManager::Zypper) => {
            "zypper --non-interactive install nodejs npm"
        }
        (RuntimeDependency::Node, LinuxPackageManager::Apk) => "apk add nodejs npm",
        (RuntimeDependency::Python, LinuxPackageManager::Apt) => {
            "apt-get update && apt-get install -y python3 python3-pip"
        }
        (RuntimeDependency::Python, LinuxPackageManager::Dnf) => {
            "dnf -y install python3 python3-pip"
        }
        (RuntimeDependency::Python, LinuxPackageManager::Yum) => {
            "yum -y install python3 python3-pip"
        }
        (RuntimeDependency::Python, LinuxPackageManager::Pacman) => {
            "pacman -Sy --noconfirm python python-pip"
        }
        (RuntimeDependency::Python, LinuxPackageManager::Zypper) => {
            "zypper --non-interactive install python3 python3-pip"
        }
        (RuntimeDependency::Python, LinuxPackageManager::Apk) => "apk add python3 py3-pip",
    }
}

fn run_shell_command_with_privilege(command: &str) -> Result<()> {
    let output = if detect_is_root_user() {
        Command::new("sh")
            .args(["-lc", command])
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?
    } else if command_exists("sudo") {
        Command::new("sudo")
            .args(["sh", "-lc", command])
            .output()
            .with_context(|| format!("Failed to execute command with sudo: {}", command))?
    } else {
        bail!(
            "Need root/sudo to install dependency, but sudo is not available. Command: {}",
            command
        );
    };

    if !output.status.success() {
        bail!(
            "Command failed: {}\nstdout: {}\nstderr: {}",
            command,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn ensure_runtime_dependency(dependency: RuntimeDependency) -> Result<()> {
    if dependency.is_available() {
        return Ok(());
    }

    let package_manager = LinuxPackageManager::detect().with_context(|| {
        format!(
            "Could not detect Linux package manager for installing '{}'",
            dependency.display_name()
        )
    })?;

    let install_command = dependency_install_command(dependency, package_manager);
    run_shell_command_with_privilege(install_command)?;

    if dependency == RuntimeDependency::Docker && command_exists("systemctl") {
        let _ = run_shell_command_with_privilege("systemctl daemon-reload");
        let _ = run_shell_command_with_privilege("systemctl enable --now docker");
    }

    if !dependency.is_available() {
        bail!(
            "Dependency '{}' still unavailable after install command",
            dependency.display_name()
        );
    }

    Ok(())
}

fn detect_runtime_dependencies(
    install_mode: UrlInstallMode,
    downloaded_file: &Path,
    executable_path: &Path,
    service_dir: &Path,
) -> BTreeSet<RuntimeDependency> {
    let mut dependencies = BTreeSet::new();

    if install_mode == UrlInstallMode::Docker {
        dependencies.insert(RuntimeDependency::Docker);
    }

    if let Some(dependency) = detect_runtime_from_path(downloaded_file) {
        dependencies.insert(dependency);
    }

    if let Some(dependency) = detect_runtime_from_path(executable_path) {
        dependencies.insert(dependency);
    }

    if directory_contains_any_file_name(
        service_dir,
        &[
            "dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ],
    ) {
        dependencies.insert(RuntimeDependency::Docker);
    }

    if directory_contains_any_file_name(
        service_dir,
        &["requirements.txt", "pyproject.toml", "setup.py", "pipfile"],
    ) {
        dependencies.insert(RuntimeDependency::Python);
    }

    if directory_contains_any_file_name(
        service_dir,
        &[
            "package.json",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
        ],
    ) {
        dependencies.insert(RuntimeDependency::Node);
    }

    dependencies
}

fn detect_runtime_from_path(path: &Path) -> Option<RuntimeDependency> {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        let lower = extension.to_ascii_lowercase();
        if lower == "py" {
            return Some(RuntimeDependency::Python);
        }
        if lower == "js" || lower == "mjs" || lower == "cjs" {
            return Some(RuntimeDependency::Node);
        }
    }

    detect_runtime_from_shebang(path)
}

fn detect_runtime_from_shebang(path: &Path) -> Option<RuntimeDependency> {
    use std::io::BufRead;

    if !path.is_file() {
        return None;
    }

    let file = std::fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).ok()?;

    if !first_line.starts_with("#!") {
        return None;
    }

    let normalized = first_line.to_ascii_lowercase();
    if normalized.contains("python") {
        return Some(RuntimeDependency::Python);
    }
    if normalized.contains("node") {
        return Some(RuntimeDependency::Node);
    }

    None
}

fn directory_contains_any_file_name(dir: &Path, candidates: &[&str]) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    let expected: Vec<String> = candidates
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();

    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_ascii_lowercase());
            if let Some(filename) = filename {
                if expected.iter().any(|candidate| candidate == &filename) {
                    return true;
                }
            }
        }
    }

    false
}
