//! API 响应模型

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// 统一 API 响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 状态码
    pub code: i32,
    /// 响应数据
    pub data: Option<T>,
    /// 消息
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// 成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            data: Some(data),
            message: "success".to_string(),
        }
    }

    /// 成功响应（带消息）
    pub fn success_with_message(data: T, message: &str) -> Self {
        Self {
            code: 0,
            data: Some(data),
            message: message.to_string(),
        }
    }

    /// 错误响应
    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            data: None,
            message: message.to_string(),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.code == 0 {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(self)).into_response()
    }
}

/// 分页响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PageResponse<T> {
    /// 数据列表
    pub items: Vec<T>,
    /// 总数
    pub total: i64,
    /// 当前页
    pub page: i32,
    /// 每页数量
    pub page_size: i32,
}

impl<T: Serialize> PageResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: i32, page_size: i32) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
        }
    }
}
