//! AI 应用模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 应用
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct App {
    pub id: i64,
    pub name: String,
    pub app_type: String,
    pub docker_compose_path: Option<String>,
    pub port: Option<i32>,
    pub status: String,
    pub config: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 应用模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTemplate {
    pub name: String,
    pub app_type: String,
    pub docker_image: String,
    pub default_port: u16,
    pub description: String,
    pub environment_vars: Vec<EnvVar>,
    pub volumes: Vec<VolumeMount>,
}

/// 环境变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub default_value: String,
    pub description: String,
    pub required: bool,
}

/// 卷挂载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
}

/// 安装应用请求
#[derive(Debug, Deserialize)]
pub struct InstallAppRequest {
    pub app_type: String,
    pub name: String,
    pub port: Option<u16>,
    pub environment_vars: std::collections::HashMap<String, String>,
}

/// 获取内置应用模板
pub fn get_app_templates() -> Vec<AppTemplate> {
    vec![
        AppTemplate {
            name: "n8n".to_string(),
            app_type: "n8n".to_string(),
            docker_image: "n8nio/n8n:latest".to_string(),
            default_port: 5678,
            description: "开源工作流自动化工具".to_string(),
            environment_vars: vec![
                EnvVar {
                    name: "N8N_BASIC_AUTH_ACTIVE".to_string(),
                    default_value: "true".to_string(),
                    description: "启用基础认证".to_string(),
                    required: false,
                },
                EnvVar {
                    name: "GENERIC_TIMEZONE".to_string(),
                    default_value: "Asia/Shanghai".to_string(),
                    description: "时区设置".to_string(),
                    required: false,
                },
            ],
            volumes: vec![
                VolumeMount {
                    host_path: "/opt/panel/data/apps/n8n".to_string(),
                    container_path: "/home/node/.n8n".to_string(),
                },
            ],
        },
        AppTemplate {
            name: "Open WebUI".to_string(),
            app_type: "openwebui".to_string(),
            docker_image: "ghcr.io/open-webui/open-webui:main".to_string(),
            default_port: 3000,
            description: "OpenAI 兼容的 Web UI".to_string(),
            environment_vars: vec![],
            volumes: vec![
                VolumeMount {
                    host_path: "/opt/panel/data/apps/openwebui".to_string(),
                    container_path: "/app/backend/data".to_string(),
                },
            ],
        },
        AppTemplate {
            name: "Dify".to_string(),
            app_type: "dify".to_string(),
            docker_image: "langgenius/dify-web:latest".to_string(),
            default_port: 8080,
            description: "LLM 应用开发平台".to_string(),
            environment_vars: vec![],
            volumes: vec![],
        },
        AppTemplate {
            name: "Qdrant".to_string(),
            app_type: "qdrant".to_string(),
            docker_image: "qdrant/qdrant:latest".to_string(),
            default_port: 6333,
            description: "向量数据库".to_string(),
            environment_vars: vec![],
            volumes: vec![
                VolumeMount {
                    host_path: "/opt/panel/data/apps/qdrant".to_string(),
                    container_path: "/qdrant/storage".to_string(),
                },
            ],
        },
    ]
}
