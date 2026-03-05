//! 注册表客户端 - 从远程源获取软件包配置

use super::{current_system_info, CachedIndex, PackageConfig, PackageIndex, PackageSummary};
use anyhow::{bail, Context, Result};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 注册表客户端配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 注册表基础 URL
    pub base_url: String,
    /// 请求超时时间
    pub timeout: Duration,
    /// 缓存目录
    pub cache_dir: PathBuf,
    /// 是否启用缓存
    pub enable_cache: bool,
}
/// 解析 cache root dir。

fn resolve_cache_root_dir() -> PathBuf {
    if let Ok(cache_dir) = std::env::var("PANEL_CACHE_DIR") {
        if !cache_dir.trim().is_empty() {
            return PathBuf::from(cache_dir);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".panel1/cache");
        }
    }

    PathBuf::from(".panel1/cache")
}
/// 解析 registry base url。

fn resolve_registry_base_url() -> String {
    if let Ok(base_url) = std::env::var("PANEL_REGISTRY_URL") {
        if !base_url.trim().is_empty() {
            return base_url;
        }
    }
    super::DEFAULT_REGISTRY_URL.to_string()
}

impl Default for RegistryConfig {
    /// 返回默认实例。
    fn default() -> Self {
        Self {
            base_url: resolve_registry_base_url(),
            timeout: Duration::from_secs(30),
            cache_dir: resolve_cache_root_dir().join("registry"),
            enable_cache: true,
        }
    }
}

/// 软件包注册表客户端
pub struct PackageRegistry {
    /// HTTP 客户端
    client: Client,
    /// 配置
    config: RegistryConfig,
    /// 内存缓存
    memory_cache: Arc<RwLock<Option<CachedIndex>>>,
}

impl PackageRegistry {
    /// 创建新的注册表客户端
    pub fn new(config: RegistryConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(format!("Panel1/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            memory_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// 使用默认配置创建客户端
    pub fn with_defaults() -> Result<Self> {
        Self::new(RegistryConfig::default())
    }

    /// 获取软件包索引
    pub async fn get_index(&self) -> Result<PackageIndex> {
        // 先检查内存缓存
        {
            let cache = self.memory_cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired() {
                    debug!("Using memory cached index");
                    return Ok(cached.index.clone());
                }
            }
        }

        // 检查磁盘缓存
        if self.config.enable_cache {
            if let Some(cached) = self.load_index_from_disk().await? {
                if !cached.is_expired() {
                    debug!("Using disk cached index");
                    // 更新内存缓存
                    let mut memory_cache = self.memory_cache.write().await;
                    *memory_cache = Some(cached.clone());
                    return Ok(cached.index);
                }
            }
        }

        // 从远程获取
        info!("Fetching package index from {}", self.config.base_url);
        let url = format!("{}/packages/index.json", self.config.base_url);
        let response = self.client.get(&url).send().await.with_context(|| {
            format!(
                "Failed to fetch package index from {}. Check network or set PANEL_REGISTRY_URL.",
                url
            )
        })?;

        if !response.status().is_success() {
            bail!(
                "Failed to fetch package index from {}: HTTP {}. Check PANEL_REGISTRY_URL.",
                url,
                response.status()
            );
        }

        let index: PackageIndex = response
            .json()
            .await
            .context("Failed to parse package index")?;

        // 缓存结果
        let cached = CachedIndex::new(index.clone());
        {
            let mut memory_cache = self.memory_cache.write().await;
            *memory_cache = Some(cached.clone());
        }

        // 写入磁盘缓存
        if self.config.enable_cache {
            if let Err(e) = self.save_index_to_disk(&cached).await {
                warn!("Failed to save index to disk: {}", e);
            }
        }

        Ok(index)
    }

    /// 获取软件包配置
    pub async fn get_package_config(&self, package_id: &str) -> Result<PackageConfig> {
        // 先尝试从磁盘缓存加载
        if self.config.enable_cache {
            if let Some(config) = self.load_package_config_from_disk(package_id).await? {
                debug!("Using disk cached config for {}", package_id);
                return Ok(config);
            }
        }

        // 先获取索引以获取 config_url
        let index = self.get_index().await?;
        let summary = index
            .packages
            .iter()
            .find(|p| p.id == package_id)
            .with_context(|| format!("Package {} not found in index", package_id))?;

        // 从远程获取配置
        let url = format!("{}/{}", self.config.base_url, summary.config_url);
        info!("Fetching package config from {}", url);

        let response = self.client.get(&url).send().await.with_context(|| {
            format!(
                "Failed to fetch package config from {}. Check network or set PANEL_REGISTRY_URL.",
                url
            )
        })?;

        if !response.status().is_success() {
            bail!(
                "Failed to fetch package config from {}: HTTP {}.",
                url,
                response.status()
            );
        }

        let config: PackageConfig = response
            .json()
            .await
            .context("Failed to parse package config")?;

        // 写入磁盘缓存
        if self.config.enable_cache {
            if let Err(e) = self.save_package_config_to_disk(&config).await {
                warn!("Failed to save package config to disk: {}", e);
            }
        }

        Ok(config)
    }

    /// 搜索软件包
    pub async fn search(&self, query: &str) -> Result<Vec<PackageSummary>> {
        let index = self.get_index().await?;
        let query_lower = query.to_lowercase();

        let results: Vec<PackageSummary> = index
            .packages
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.id.to_lowercase().contains(&query_lower)
                    || p.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    /// 按分类列出软件包
    pub async fn list_by_category(
        &self,
    ) -> Result<std::collections::HashMap<super::PackageCategory, Vec<PackageSummary>>> {
        let index = self.get_index().await?;
        let mut result = std::collections::HashMap::new();

        for package in index.packages {
            result
                .entry(package.category.clone())
                .or_insert_with(Vec::new)
                .push(package);
        }

        Ok(result)
    }

    /// 查找适合当前平台的二进制文件
    pub fn find_artifact(
        &self,
        config: &PackageConfig,
        version: &str,
    ) -> Result<Option<super::Artifact>> {
        let (os, arch) = current_system_info();

        let version_info = config
            .versions
            .iter()
            .find(|v| v.version == version)
            .with_context(|| format!("Version {} not found", version))?;

        for artifact in &version_info.artifacts {
            if artifact.os == os && artifact.arch == arch {
                return Ok(Some(artifact.clone()));
            }
        }

        Ok(None)
    }

    /// 列出软件包的所有版本
    pub async fn list_versions(&self, package_id: &str) -> Result<Vec<String>> {
        let config = self.get_package_config(package_id).await?;
        Ok(config.versions.iter().map(|v| v.version.clone()).collect())
    }

    /// 获取 LTS 版本
    pub async fn get_lts_versions(&self, package_id: &str) -> Result<Vec<String>> {
        let config = self.get_package_config(package_id).await?;
        Ok(config
            .versions
            .iter()
            .filter(|v| v.lts)
            .map(|v| v.version.clone())
            .collect())
    }

    /// 清除缓存
    pub async fn clear_cache(&self) -> Result<()> {
        // 清除内存缓存
        {
            let mut memory_cache = self.memory_cache.write().await;
            *memory_cache = None;
        }

        // 清除磁盘缓存
        if self.config.cache_dir.exists() {
            tokio::fs::remove_dir_all(&self.config.cache_dir)
                .await
                .context("Failed to remove cache directory")?;
        }

        info!("Registry cache cleared");
        Ok(())
    }

    /// 从磁盘加载索引缓存
    async fn load_index_from_disk(&self) -> Result<Option<CachedIndex>> {
        let cache_file = self.config.cache_dir.join("index.json");

        if !cache_file.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&cache_file)
            .await
            .context("Failed to read cached index")?;

        let index: PackageIndex =
            serde_json::from_str(&content).context("Failed to parse cached index")?;

        // 使用文件修改时间作为缓存时间
        let metadata = tokio::fs::metadata(&cache_file)
            .await
            .context("Failed to read cache file metadata")?;

        let modified = metadata
            .modified()
            .context("Failed to get cache file modification time")?;

        Ok(Some(CachedIndex {
            index,
            cached_at: modified,
        }))
    }

    /// 保存索引到磁盘
    async fn save_index_to_disk(&self, cached: &CachedIndex) -> Result<()> {
        tokio::fs::create_dir_all(&self.config.cache_dir)
            .await
            .context("Failed to create cache directory")?;

        let cache_file = self.config.cache_dir.join("index.json");
        let content =
            serde_json::to_string_pretty(&cached.index).context("Failed to serialize index")?;

        tokio::fs::write(&cache_file, content)
            .await
            .context("Failed to write index cache")?;

        debug!("Index cached to {:?}", cache_file);
        Ok(())
    }

    /// 从磁盘加载软件包配置缓存
    async fn load_package_config_from_disk(
        &self,
        package_id: &str,
    ) -> Result<Option<PackageConfig>> {
        let cache_file = self
            .config
            .cache_dir
            .join("packages")
            .join(format!("{}.json", package_id));

        if !cache_file.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&cache_file)
            .await
            .context("Failed to read cached package config")?;

        let config: PackageConfig =
            serde_json::from_str(&content).context("Failed to parse cached package config")?;

        Ok(Some(config))
    }

    /// 保存软件包配置到磁盘
    async fn save_package_config_to_disk(&self, config: &PackageConfig) -> Result<()> {
        let packages_dir = self.config.cache_dir.join("packages");
        tokio::fs::create_dir_all(&packages_dir)
            .await
            .context("Failed to create packages cache directory")?;

        let cache_file = packages_dir.join(format!("{}.json", config.id));
        let content =
            serde_json::to_string_pretty(config).context("Failed to serialize package config")?;

        tokio::fs::write(&cache_file, content)
            .await
            .context("Failed to write package config cache")?;

        debug!("Package config cached to {:?}", cache_file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：验证 registry config default。
    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();
        assert_eq!(config.base_url, super::super::DEFAULT_REGISTRY_URL);
        assert!(config.enable_cache);
    }

    /// 测试：验证 find artifact。
    #[tokio::test]
    async fn test_find_artifact() {
        let registry = PackageRegistry::with_defaults().unwrap();

        let config = PackageConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test package".to_string(),
            category: super::super::PackageCategory::Tool,
            homepage: None,
            documentation: None,
            versions: vec![super::super::PackageVersion {
                version: "1.0.0".to_string(),
                lts: false,
                stable: true,
                release_date: None,
                artifacts: vec![super::super::Artifact {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                    url: "https://example.com/test.tar.gz".to_string(),
                    checksum: super::super::Checksum {
                        algorithm: "sha256".to_string(),
                        value: "abc123".to_string(),
                    },
                    size: None,
                    archive_format: "tar.gz".to_string(),
                }],
                changelog: None,
            }],
            install: super::super::InstallConfig {
                binary: None,
                pre_install: None,
                post_install: None,
                directories: vec![],
                environment: std::collections::HashMap::new(),
            },
            service: None,
        };

        let result = registry.find_artifact(&config, "1.0.0");
        // 结果取决于当前系统
        assert!(result.is_ok());
    }
}
