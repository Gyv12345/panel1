//! Docker 管理 API 路由

use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::ApiResponse;
use crate::services::docker::DockerService;

lazy_static::lazy_static! {
    static ref DOCKER_SERVICE: Arc<Mutex<DockerService>> =
        Arc::new(Mutex::new(DockerService::new()));
}

/// 创建容器请求
#[derive(Debug, Deserialize)]
pub struct CreateContainerRequest {
    pub name: String,
    pub image: String,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<PortMappingRequest>>,
    pub volumes: Option<Vec<VolumeMappingRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct PortMappingRequest {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VolumeMappingRequest {
    pub host_path: String,
    pub container_path: String,
}

/// 拉取镜像请求
#[derive(Debug, Deserialize)]
pub struct PullImageRequest {
    pub image: String,
}

/// 获取容器列表
pub async fn list_containers() -> Json<ApiResponse<Vec<panel_docker::ContainerInfo>>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.list_containers(true).await {
        Ok(containers) => Json(ApiResponse::success(containers)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 创建容器 (简化版)
pub async fn create_container(
    Json(_req): Json<CreateContainerRequest>,
) -> Json<ApiResponse<String>> {
    // TODO: 实现容器创建
    Json(ApiResponse::error(501, "Container creation not implemented yet"))
}

/// 启动容器
pub async fn start_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<()>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.container_manager().await {
        Ok(manager) => {
            match manager.start_container(&id).await {
                Ok(()) => Json(ApiResponse::success_with_message((), "Container started")),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 停止容器
pub async fn stop_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<()>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.container_manager().await {
        Ok(manager) => {
            match manager.stop_container(&id, None).await {
                Ok(()) => Json(ApiResponse::success_with_message((), "Container stopped")),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 重启容器
pub async fn restart_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<()>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.container_manager().await {
        Ok(manager) => {
            match manager.restart_container(&id, None).await {
                Ok(()) => Json(ApiResponse::success_with_message((), "Container restarted")),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 删除容器
pub async fn remove_container(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<()>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.container_manager().await {
        Ok(manager) => {
            match manager.remove_container(&id, true, false).await {
                Ok(()) => Json(ApiResponse::success_with_message((), "Container removed")),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 获取容器日志
pub async fn container_logs(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<String>>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.container_manager().await {
        Ok(manager) => {
            match manager.get_container_logs(&id, Some(100)).await {
                Ok(logs) => Json(ApiResponse::success(logs)),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 获取镜像列表
pub async fn list_images() -> Json<ApiResponse<Vec<panel_docker::ImageInfo>>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.list_images().await {
        Ok(images) => Json(ApiResponse::success(images)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 拉取镜像
pub async fn pull_image(
    Json(req): Json<PullImageRequest>,
) -> Json<ApiResponse<Vec<panel_docker::PullProgress>>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.image_manager().await {
        Ok(manager) => {
            match manager.pull_image(&req.image).await {
                Ok(progress) => Json(ApiResponse::success(progress)),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 删除镜像
pub async fn remove_image(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<()>> {
    let mut service = DOCKER_SERVICE.lock().await;

    match service.image_manager().await {
        Ok(manager) => {
            match manager.remove_image(&id, false).await {
                Ok(()) => Json(ApiResponse::success_with_message((), "Image removed")),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
