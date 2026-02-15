//! AI 应用管理 API 路由

use axum::{
    extract::{Extension, Path},
    Json,
};
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::models::{ApiResponse, App, AppTemplate, InstallAppRequest};
use crate::services::apps::AppService;

/// 获取应用服务
fn get_app_service(db_pool: SqlitePool) -> AppService {
    let data_dir = std::env::var("PANEL_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/panel/data"));
    AppService::new(db_pool, &data_dir)
}

/// 获取已安装应用列表
pub async fn list(
    Extension(db_pool): Extension<SqlitePool>,
) -> Json<ApiResponse<Vec<App>>> {
    let service = get_app_service(db_pool);

    match service.list().await {
        Ok(apps) => Json(ApiResponse::success(apps)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 获取应用模板列表
pub async fn templates() -> Json<ApiResponse<Vec<AppTemplate>>> {
    Json(ApiResponse::success(AppService::get_templates()))
}

/// 安装应用
pub async fn install(
    Extension(db_pool): Extension<SqlitePool>,
    Json(req): Json<InstallAppRequest>,
) -> Json<ApiResponse<App>> {
    let service = get_app_service(db_pool);

    match service.install(&req).await {
        Ok(app) => Json(ApiResponse::success(app)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 启动应用
pub async fn start(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let service = get_app_service(db_pool);

    match service.start(id).await {
        Ok(()) => Json(ApiResponse::success_with_message((), "App started")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 停止应用
pub async fn stop(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let service = get_app_service(db_pool);

    match service.stop(id).await {
        Ok(()) => Json(ApiResponse::success_with_message((), "App stopped")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 卸载应用
pub async fn uninstall(
    Extension(db_pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let service = get_app_service(db_pool);

    match service.uninstall(id).await {
        Ok(()) => Json(ApiResponse::success_with_message((), "App uninstalled")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
