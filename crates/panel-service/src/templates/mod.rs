//! 服务模板 - 预定义的服务配置模板

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 服务模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTemplate {
    /// 模板 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 服务类型
    pub service_type: String,
    /// 默认版本
    pub default_version: String,
    /// 可用版本列表
    pub available_versions: Vec<String>,
    /// 默认端口
    pub default_port: u16,
    /// 下载 URL 模板
    pub download_url_template: String,
    /// 配置文件模板
    pub config_template: Option<String>,
    /// 环境变量模板
    pub env_template: Option<HashMap<String, String>>,
    /// 启动参数模板
    pub start_args: Option<Vec<String>>,
}

/// 模板注册表
pub struct TemplateRegistry {
    templates: HashMap<String, ServiceTemplate>,
}

impl TemplateRegistry {
    /// 创建新的模板注册表
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        registry.register_builtin_templates();
        registry
    }

    /// 注册内置模板
    fn register_builtin_templates(&mut self) {
        // Node.js 模板
        self.register(ServiceTemplate {
            id: "nodejs".to_string(),
            name: "Node.js".to_string(),
            description: "JavaScript 运行时环境".to_string(),
            service_type: "nodejs".to_string(),
            default_version: "22.14.0".to_string(),
            available_versions: vec![
                "24.0.0".to_string(),  // Current
                "23.11.0".to_string(),
                "22.14.0".to_string(), // LTS
                "22.13.0".to_string(),
                "20.19.0".to_string(), // LTS
                "20.18.0".to_string(),
            ],
            default_port: 0, // Node.js 应用通常不需要固定端口
            download_url_template:
                "https://nodejs.org/dist/v{version}/node-v{version}-linux-x64.tar.xz".to_string(),
            config_template: None,
            env_template: None,
            start_args: None,
        });

        // Redis 模板
        self.register(ServiceTemplate {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            description: "高性能内存键值数据库".to_string(),
            service_type: "redis".to_string(),
            default_version: "7.2".to_string(),
            available_versions: vec!["7.2".to_string(), "7.0".to_string(), "6.2".to_string()],
            default_port: 6379,
            download_url_template:
                "https://github.com/redis/redis/archive/refs/tags/{version}.tar.gz".to_string(),
            config_template: Some(include_str!("redis.conf.template").to_string()),
            env_template: None,
            start_args: Some(vec!["--port".to_string(), "{port}".to_string()]),
        });

        // Elasticsearch 模板
        self.register(ServiceTemplate {
            id: "elasticsearch".to_string(),
            name: "Elasticsearch".to_string(),
            description: "分布式搜索和分析引擎".to_string(),
            service_type: "elasticsearch".to_string(),
            default_version: "8.11".to_string(),
            available_versions: vec!["8.11".to_string(), "8.10".to_string(), "7.17".to_string()],
            default_port: 9200,
            download_url_template: "https://artifacts.elastic.co/downloads/elasticsearch/elasticsearch-{version}-linux-x86_64.tar.gz".to_string(),
            config_template: None,
            env_template: Some({
                let mut env = HashMap::new();
                env.insert("ES_JAVA_OPTS".to_string(), "-Xms512m -Xmx512m".to_string());
                env
            }),
            start_args: None,
        });

        // PostgreSQL 模板
        self.register(ServiceTemplate {
            id: "postgresql".to_string(),
            name: "PostgreSQL".to_string(),
            description: "强大的开源关系数据库".to_string(),
            service_type: "postgresql".to_string(),
            default_version: "16".to_string(),
            available_versions: vec!["16".to_string(), "15".to_string(), "14".to_string()],
            default_port: 5432,
            download_url_template:
                "https://ftp.postgresql.org/pub/source/v{version}/postgresql-{version}.tar.gz"
                    .to_string(),
            config_template: None,
            env_template: None,
            start_args: None,
        });

        // Nginx 模板
        self.register(ServiceTemplate {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            description: "高性能 HTTP 和反向代理服务器".to_string(),
            service_type: "nginx".to_string(),
            default_version: "1.24".to_string(),
            available_versions: vec!["1.24".to_string(), "1.23".to_string(), "1.22".to_string()],
            default_port: 80,
            download_url_template: "https://nginx.org/download/nginx-{version}.tar.gz".to_string(),
            config_template: Some(include_str!("nginx.conf.template").to_string()),
            env_template: None,
            start_args: None,
        });
    }

    /// 注册模板
    pub fn register(&mut self, template: ServiceTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// 获取模板
    pub fn get(&self, id: &str) -> Option<&ServiceTemplate> {
        self.templates.get(id)
    }

    /// 获取所有模板
    pub fn list(&self) -> Vec<&ServiceTemplate> {
        self.templates.values().collect()
    }

    /// 检查模板是否存在
    pub fn exists(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
