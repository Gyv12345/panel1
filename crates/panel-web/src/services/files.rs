//! 文件管理服务

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// 文件/目录信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub permissions: String,
}

/// 文件管理服务
pub struct FileService {
    base_path: PathBuf,
}

impl FileService {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    /// 列出目录内容
    pub fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>> {
        let full_path = self.resolve_path(path)?;

        if !full_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if !full_path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path);
        }

        let entries = std::fs::read_dir(&full_path)
            .with_context(|| format!("Failed to read directory: {}", path))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;

            let file_info = FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified()
                    .ok()
                    .map(|t| {
                        let datetime: chrono::DateTime<chrono::Utc> = t.into();
                        datetime.to_rfc3339()
                    }),
                permissions: format!("{:?}", metadata.permissions()),
            };

            files.push(file_info);
        }

        // 排序：目录在前，然后按名称排序
        files.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(files)
    }

    /// 读取文件内容
    pub fn read_file(&self, path: &str) -> Result<String> {
        let full_path = self.resolve_path(path)?;

        if !full_path.exists() {
            anyhow::bail!("File does not exist: {}", path);
        }

        if full_path.is_dir() {
            anyhow::bail!("Path is a directory: {}", path);
        }

        let content = std::fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        Ok(content)
    }

    /// 写入文件内容
    pub fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, content)
            .with_context(|| format!("Failed to write file: {}", path))?;

        Ok(())
    }

    /// 创建目录
    pub fn create_directory(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;

        std::fs::create_dir_all(&full_path)
            .with_context(|| format!("Failed to create directory: {}", path))?;

        Ok(())
    }

    /// 创建文件
    pub fn create_file(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::File::create(&full_path)
            .with_context(|| format!("Failed to create file: {}", path))?;

        Ok(())
    }

    /// 删除文件或目录
    pub fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;

        if !full_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path)
                .with_context(|| format!("Failed to delete directory: {}", path))?;
        } else {
            std::fs::remove_file(&full_path)
                .with_context(|| format!("Failed to delete file: {}", path))?;
        }

        Ok(())
    }

    /// 重命名/移动文件或目录
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        let old_full = self.resolve_path(old_path)?;
        let new_full = self.resolve_path(new_path)?;

        if !old_full.exists() {
            anyhow::bail!("Source path does not exist: {}", old_path);
        }

        std::fs::rename(&old_full, &new_full)
            .with_context(|| format!("Failed to rename: {} -> {}", old_path, new_path))?;

        Ok(())
    }

    /// 解析路径（防止目录遍历攻击）
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let path = path.trim_start_matches('/');

        let full_path = if path.is_empty() {
            self.base_path.clone()
        } else {
            self.base_path.join(path)
        };

        // 规范化路径
        let canonical = if full_path.exists() {
            full_path.canonicalize()?
        } else {
            full_path.clone()
        };

        // 检查是否在基础路径内
        if !canonical.starts_with(&self.base_path) {
            anyhow::bail!("Access denied: path outside allowed directory");
        }

        Ok(canonical)
    }
}
