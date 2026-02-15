//! Docker 镜像管理

use bollard::image::{ListImagesOptions, CreateImageOptions, RemoveImageOptions};
use bollard::models::ImageSummary;
use bollard::Docker;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

/// 镜像信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// 镜像 ID
    pub id: String,
    /// 镜像名称列表 (包含 tag)
    pub repo_tags: Vec<String>,
    /// 镜像大小 (字节)
    pub size: u64,
    /// 创建时间
    pub created: i64,
}

/// 镜像拉取进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    /// 镜像 ID
    pub id: String,
    /// 状态
    pub status: String,
    /// 进度详情
    pub progress: Option<String>,
}

/// Docker 镜像管理器
pub struct ImageManager {
    docker: Docker,
}

impl ImageManager {
    /// 创建新的镜像管理器
    pub async fn new() -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self { docker })
    }

    /// 获取所有镜像列表
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>, anyhow::Error> {
        let options = ListImagesOptions::<String> {
            ..Default::default()
        };

        let images = self.docker.list_images(Some(options)).await?;

        Ok(images.into_iter().map(|i| self.image_summary_to_info(i)).collect())
    }

    /// 拉取镜像
    pub async fn pull_image(&self, image_name: &str) -> Result<Vec<PullProgress>, anyhow::Error> {
        let options = CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        };

        let mut progress_list = Vec::new();
        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    let progress = PullProgress {
                        id: info.id.unwrap_or_default(),
                        status: info.status.unwrap_or_default(),
                        progress: info.progress,
                    };
                    progress_list.push(progress);
                }
                Err(e) => {
                    tracing::warn!("Pull image error: {}", e);
                    break;
                }
            }
        }

        Ok(progress_list)
    }

    /// 删除镜像
    pub async fn remove_image(&self, image_id: &str, force: bool) -> Result<(), anyhow::Error> {
        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };

        self.docker.remove_image(image_id, Some(options), None).await?;
        Ok(())
    }

    /// 转换镜像摘要到信息
    fn image_summary_to_info(&self, summary: ImageSummary) -> ImageInfo {
        ImageInfo {
            id: summary.id,
            repo_tags: summary.repo_tags,
            size: summary.size as u64,
            created: summary.created,
        }
    }
}
