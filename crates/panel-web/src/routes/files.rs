//! 文件管理 API 路由

use axum::{
    extract::{Extension, Query},
    Json,
    body::Body,
    http::{header, Response, StatusCode},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::models::ApiResponse;
use crate::services::files::{FileService, FileInfo};

/// 文件列表请求
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
}

/// 文件内容请求
#[derive(Debug, Deserialize)]
pub struct ContentQuery {
    pub path: String,
}

/// 创建文件/目录请求
#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String, // "file" or "directory"
}

/// 更新文件请求
#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub path: String,
    pub content: String,
}

/// 删除文件请求
#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

/// 获取文件服务
fn get_file_service() -> FileService {
    let base_path = std::env::var("PANEL_FILE_ROOT")
        .unwrap_or_else(|_| "/".to_string());
    FileService::new(&PathBuf::from(&base_path))
}

/// 列出目录内容
pub async fn list(
    Query(query): Query<ListQuery>,
) -> Json<ApiResponse<Vec<FileInfo>>> {
    let service = get_file_service();
    let path = query.path.unwrap_or_default();

    match service.list_directory(&path) {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 获取文件内容
pub async fn get_content(
    Query(query): Query<ContentQuery>,
) -> Json<ApiResponse<String>> {
    let service = get_file_service();

    match service.read_file(&query.path) {
        Ok(content) => Json(ApiResponse::success(content)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 创建文件/目录
pub async fn create(
    Json(req): Json<CreateRequest>,
) -> Json<ApiResponse<()>> {
    let service = get_file_service();

    let result = match req.file_type.as_str() {
        "file" => service.create_file(&req.path),
        "directory" | "dir" => service.create_directory(&req.path),
        _ => return Json(ApiResponse::error(400, "Invalid type. Use 'file' or 'directory'")),
    };

    match result {
        Ok(()) => Json(ApiResponse::success_with_message((), "Created successfully")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 更新文件内容
pub async fn update(
    Json(req): Json<UpdateRequest>,
) -> Json<ApiResponse<()>> {
    let service = get_file_service();

    match service.write_file(&req.path, &req.content) {
        Ok(()) => Json(ApiResponse::success_with_message((), "Updated successfully")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 删除文件/目录
pub async fn delete_(
    Query(query): Query<DeleteQuery>,
) -> Json<ApiResponse<()>> {
    let service = get_file_service();

    match service.delete(&query.path) {
        Ok(()) => Json(ApiResponse::success_with_message((), "Deleted successfully")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 上传文件 (简化版)
pub async fn upload(
    Query(query): Query<ContentQuery>,
) -> Json<ApiResponse<()>> {
    // TODO: 实现文件上传
    Json(ApiResponse::error(501, "File upload not implemented yet"))
}

/// 下载文件
pub async fn download(
    Query(query): Query<ContentQuery>,
) -> Response<Body> {
    let service = get_file_service();

    match service.read_file(&query.path) {
        Ok(content) => {
            let file_name = query.path.rsplit('/').next().unwrap_or("download");
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name))
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .body(Body::from(content))
                .unwrap()
        }
        Err(e) => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(e.to_string()))
                .unwrap()
        }
    }
}
