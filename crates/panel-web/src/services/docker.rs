//! Docker 服务

use panel_docker::{ContainerManager, ImageManager, ContainerInfo, ImageInfo};

/// Docker 服务
pub struct DockerService {
    container_manager: Option<ContainerManager>,
    image_manager: Option<ImageManager>,
}

impl DockerService {
    pub fn new() -> Self {
        Self {
            container_manager: None,
            image_manager: None,
        }
    }

    /// 初始化容器管理器
    pub async fn container_manager(&mut self) -> Result<&ContainerManager, anyhow::Error> {
        if self.container_manager.is_none() {
            self.container_manager = Some(ContainerManager::new().await?);
        }
        Ok(self.container_manager.as_ref().unwrap())
    }

    /// 初始化镜像管理器
    pub async fn image_manager(&mut self) -> Result<&ImageManager, anyhow::Error> {
        if self.image_manager.is_none() {
            self.image_manager = Some(ImageManager::new().await?);
        }
        Ok(self.image_manager.as_ref().unwrap())
    }

    /// 获取容器列表
    pub async fn list_containers(&mut self, all: bool) -> Result<Vec<ContainerInfo>, anyhow::Error> {
        let manager = self.container_manager().await?;
        manager.list_containers(all).await
    }

    /// 获取镜像列表
    pub async fn list_images(&mut self) -> Result<Vec<ImageInfo>, anyhow::Error> {
        let manager = self.image_manager().await?;
        manager.list_images().await
    }
}

impl Default for DockerService {
    fn default() -> Self {
        Self::new()
    }
}
