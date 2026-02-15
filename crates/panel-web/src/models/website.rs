//! 网站模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 网站
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Website {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub root_path: String,
    pub port: i32,
    pub ssl_enabled: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
    pub nginx_config: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// 创建网站请求
#[derive(Debug, Deserialize)]
pub struct CreateWebsiteRequest {
    pub name: String,
    pub domain: String,
    pub root_path: String,
    pub port: Option<i32>,
}

/// 更新网站请求
#[derive(Debug, Deserialize)]
pub struct UpdateWebsiteRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub root_path: Option<String>,
    pub port: Option<i32>,
}

/// SSL 配置请求
#[derive(Debug, Deserialize)]
pub struct SslConfigRequest {
    pub cert_path: String,
    pub key_path: String,
}
