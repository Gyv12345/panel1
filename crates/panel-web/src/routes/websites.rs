//! 网站管理 API 路由

use axum::{
    extract::{Extension, Path},
    Json,
};
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::models::{
    ApiResponse, CreateWebsiteRequest, UpdateWebsiteRequest, SslConfigRequest, Website,
};
use crate::services::websites::WebsiteService;

/// 获取网站服务
fn get_website_service(db_pool: SqlitePool) -> WebsiteService {
    let config_dir = std::env::var("PANEL_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/panel/config"));
    WebsiteService::new(db_pool, &config_dir)
}

/// 获取网站列表
pub async fn list(
    Extension(db_pool): Extension<SqlitePool>,
) -> Json<ApiResponse<Vec<Website>>> {
    let service = get_website_service(db_pool);

    match service.list().await {
        Ok(websites) => Json(ApiResponse::success(websites)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 创建网站
pub async fn create(
    Extension(db_pool): Extension<SqlitePool>,
    Json(req): Json<CreateWebsiteRequest>,
) -> Json<ApiResponse<Website>> {
    let service = get_website_service(db_pool);

    match service.create(&req).await {
        Ok(website) => Json(ApiResponse::success(website)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 更新网站
pub async fn update(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWebsiteRequest>,
) -> Json<ApiResponse<Website>> {
    let service = get_website_service(db_pool);

    match service.update(id, &req).await {
        Ok(website) => Json(ApiResponse::success(website)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 删除网站
pub async fn delete_(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let service = get_website_service(db_pool);

    match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success_with_message((), "Website deleted")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 配置 SSL
pub async fn configure_ssl(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<SslConfigRequest>,
) -> Json<ApiResponse<Website>> {
    let service = get_website_service(db_pool);

    match service.configure_ssl(id, &req).await {
        Ok(website) => Json(ApiResponse::success(website)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 重载 Nginx
pub async fn reload_nginx(
    Extension(db_pool): Extension<SqlitePool>,
    Path(_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let service = get_website_service(db_pool);

    match service.reload_nginx().await {
        Ok(()) => Json(ApiResponse::success_with_message((), "Nginx reloaded")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
