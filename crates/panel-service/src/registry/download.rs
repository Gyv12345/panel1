//! 下载管理器 - 下载和校验二进制文件

use super::Artifact;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256, Sha512};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 下载进度回调
pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync>;

/// 下载进度信息
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// 已下载字节数
    pub downloaded: u64,
    /// 总字节数（如果已知）
    pub total: Option<u64>,
    /// 下载速度（字节/秒）
    pub speed: Option<u64>,
    /// 百分比
    pub percent: Option<f64>,
}

impl DownloadProgress {
    /// 创建新的进度信息
    pub fn new(downloaded: u64, total: Option<u64>) -> Self {
        let percent = total.map(|t| {
            if t > 0 {
                (downloaded as f64 / t as f64) * 100.0
            } else {
                0.0
            }
        });

        Self {
            downloaded,
            total,
            speed: None,
            percent,
        }
    }

    /// 格式化为可读字符串
    pub fn format(&self) -> String {
        let downloaded = format_size(self.downloaded);
        let total = self
            .total
            .map(format_size)
            .unwrap_or_else(|| "未知".to_string());
        let percent = self
            .percent
            .map(|p| format!("{:.1}%", p))
            .unwrap_or_else(|| "计算中".to_string());

        format!("{} / {} ({})", downloaded, total, percent)
    }
}

/// 格式化字节大小为可读字符串
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 下载管理器配置
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 下载缓存目录
    pub cache_dir: PathBuf,
    /// 下载超时时间（秒）
    pub timeout: u64,
    /// 块大小（字节）
    pub chunk_size: usize,
    /// 最大重试次数
    pub max_retries: u32,
}

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

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            cache_dir: resolve_cache_root_dir().join("downloads"),
            timeout: 300,
            chunk_size: 8192,
            max_retries: 3,
        }
    }
}

/// 下载管理器
pub struct DownloadManager {
    /// HTTP 客户端
    client: reqwest::Client,
    /// 配置
    config: DownloadConfig,
    /// 进度回调
    progress_callback: RwLock<Option<ProgressCallback>>,
    /// 取消标志
    cancelled: Arc<RwLock<bool>>,
}

impl DownloadManager {
    /// 创建新的下载管理器
    pub fn new(config: DownloadConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .user_agent(format!("Panel1/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            progress_callback: RwLock::new(None),
            cancelled: Arc::new(RwLock::new(false)),
        })
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Result<Self> {
        Self::new(DownloadConfig::default())
    }

    /// 设置进度回调
    pub async fn set_progress_callback(&self, callback: Option<ProgressCallback>) {
        let mut cb = self.progress_callback.write().await;
        *cb = callback;
    }

    /// 取消当前下载
    pub async fn cancel(&self) {
        let mut cancelled = self.cancelled.write().await;
        *cancelled = true;
    }

    /// 下载文件
    pub async fn download(&self, artifact: &Artifact, target_path: &Path) -> Result<PathBuf> {
        // 重置取消标志
        {
            let mut cancelled = self.cancelled.write().await;
            *cancelled = false;
        }

        // 确保缓存目录存在
        tokio::fs::create_dir_all(&self.config.cache_dir)
            .await
            .context("Failed to create download cache directory")?;

        // 生成本地缓存文件名
        let cache_filename = generate_cache_filename(&artifact.url);
        let cache_path = self.config.cache_dir.join(&cache_filename);

        // 检查是否已有缓存
        if cache_path.exists() {
            debug!("Found cached file: {:?}", cache_path);
            // 验证缓存文件的校验和
            if self
                .verify_checksum(&cache_path, &artifact.checksum)
                .await?
            {
                info!("Using cached file (checksum verified): {:?}", cache_path);
                // 复制到目标路径
                if cache_path != target_path {
                    tokio::fs::copy(&cache_path, target_path)
                        .await
                        .context("Failed to copy cached file to target")?;
                }
                return Ok(target_path.to_path_buf());
            } else {
                warn!("Cached file checksum mismatch, re-downloading");
                let _ = tokio::fs::remove_file(&cache_path).await;
            }
        }

        // 下载文件
        info!("Downloading {} to {:?}", artifact.url, cache_path);

        let mut retries = 0;
        loop {
            match self
                .download_file(&artifact.url, &cache_path, artifact.size)
                .await
            {
                Ok(_) => break,
                Err(e) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        bail!("Download failed after {} retries: {}", retries, e);
                    }
                    warn!(
                        "Download failed (attempt {}/{}): {}. Retrying...",
                        retries, self.config.max_retries, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        // 验证下载的文件
        if !self
            .verify_checksum(&cache_path, &artifact.checksum)
            .await?
        {
            bail!("Downloaded file checksum mismatch");
        }

        info!("Download complete and verified: {:?}", cache_path);

        // 复制到目标路径
        if cache_path != target_path {
            tokio::fs::copy(&cache_path, target_path)
                .await
                .context("Failed to copy downloaded file to target")?;
        }

        Ok(target_path.to_path_buf())
    }

    /// 下载文件实现
    async fn download_file(
        &self,
        url: &str,
        path: &Path,
        expected_size: Option<u64>,
    ) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to start download")?;

        if !response.status().is_success() {
            bail!("Download failed: HTTP {}", response.status());
        }

        let total_size = response.content_length().or(expected_size);
        let mut downloaded: u64 = 0;

        // 创建临时文件
        let temp_path = path.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .context("Failed to create temporary file")?;

        // 流式下载
        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        let start_time = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            // 检查是否被取消
            {
                let cancelled = self.cancelled.read().await;
                if *cancelled {
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    bail!("Download cancelled");
                }
            }

            let chunk = chunk_result.context("Failed to read download chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write download chunk")?;

            downloaded += chunk.len() as u64;

            // 计算速度
            let elapsed = start_time.elapsed().as_secs();
            let speed = if elapsed > 0 {
                Some(downloaded / elapsed)
            } else {
                None
            };

            // 报告进度
            let callback = self.progress_callback.read().await;
            if let Some(ref cb) = *callback {
                let mut progress = DownloadProgress::new(downloaded, total_size);
                progress.speed = speed;
                cb(progress);
            }
        }

        file.flush().await.context("Failed to flush file")?;
        drop(file);

        // 重命名临时文件到目标文件
        tokio::fs::rename(&temp_path, path)
            .await
            .context("Failed to rename temporary file")?;

        Ok(())
    }

    /// 验证校验和
    pub async fn verify_checksum(&self, path: &Path, checksum: &super::Checksum) -> Result<bool> {
        if !path.exists() {
            bail!("File does not exist: {:?}", path);
        }

        let content = tokio::fs::read(path)
            .await
            .context("Failed to read file for checksum verification")?;

        let computed = match checksum.algorithm.to_lowercase().as_str() {
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                hex::encode(hasher.finalize())
            }
            "sha512" => {
                let mut hasher = Sha512::new();
                hasher.update(&content);
                hex::encode(hasher.finalize())
            }
            "md5" => {
                // MD5 不推荐使用，但为了兼容性仍然支持
                use md5::{Digest as Md5Digest, Md5};
                let mut hasher = Md5::new();
                hasher.update(&content);
                hex::encode(hasher.finalize())
            }
            _ => {
                warn!("Unknown checksum algorithm: {}", checksum.algorithm);
                return Ok(false);
            }
        };

        let computed_lower = computed.to_lowercase();
        let expected_lower = checksum.value.to_lowercase();

        Ok(computed_lower == expected_lower)
    }

    /// 清除下载缓存
    pub async fn clear_cache(&self) -> Result<()> {
        if self.config.cache_dir.exists() {
            tokio::fs::remove_dir_all(&self.config.cache_dir)
                .await
                .context("Failed to remove download cache")?;

            tokio::fs::create_dir_all(&self.config.cache_dir)
                .await
                .context("Failed to recreate download cache directory")?;
        }

        info!("Download cache cleared");
        Ok(())
    }

    /// 获取缓存大小
    pub async fn get_cache_size(&self) -> Result<u64> {
        if !self.config.cache_dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0;
        let mut entries = tokio::fs::read_dir(&self.config.cache_dir)
            .await
            .context("Failed to read cache directory")?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read cache entry")?
        {
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
}

/// 生成缓存文件名
fn generate_cache_filename(url: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hex::encode(hasher.finalize());

    // 尝试从 URL 中提取文件扩展名
    let extension = url
        .rsplit('.')
        .next()
        .and_then(|ext| {
            if ext.len() <= 10 && ext.chars().all(|c| c.is_alphanumeric() || c == '-') {
                Some(ext)
            } else {
                None
            }
        })
        .unwrap_or("bin");

    format!("{}.{}", hash, extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress::new(500, Some(1000));
        assert_eq!(progress.downloaded, 500);
        assert_eq!(progress.total, Some(1000));
        assert_eq!(progress.percent, Some(50.0));
    }

    #[test]
    fn test_generate_cache_filename() {
        let url = "https://example.com/file.tar.gz";
        let filename = generate_cache_filename(url);
        assert!(filename.ends_with(".gz"));
        assert!(filename.len() > 64); // SHA256 hash + extension
    }
}
