//! 网站管理服务

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{Website, CreateWebsiteRequest, UpdateWebsiteRequest, SslConfigRequest};

/// 网站管理服务
pub struct WebsiteService {
    db_pool: SqlitePool,
    nginx_config_dir: std::path::PathBuf,
}

impl WebsiteService {
    pub fn new(db_pool: SqlitePool, config_dir: &std::path::Path) -> Self {
        Self {
            db_pool,
            nginx_config_dir: config_dir.join("nginx"),
        }
    }

    /// 获取所有网站
    pub async fn list(&self) -> Result<Vec<Website>> {
        let websites = sqlx::query_as::<_, Website>(
            "SELECT id, name, domain, root_path, port, ssl_enabled, ssl_cert_path, ssl_key_path, nginx_config, status, created_at FROM websites ORDER BY created_at DESC"
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(websites)
    }

    /// 根据 ID 获取网站
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Website>> {
        let website = sqlx::query_as::<_, Website>(
            "SELECT id, name, domain, root_path, port, ssl_enabled, ssl_cert_path, ssl_key_path, nginx_config, status, created_at FROM websites WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(website)
    }

    /// 创建网站
    pub async fn create(&self, req: &CreateWebsiteRequest) -> Result<Website> {
        let port = req.port.unwrap_or(80);
        let now = Utc::now();
        let nginx_config = self.generate_nginx_config(&req.name, &req.domain, &req.root_path, port, false)?;

        let result = sqlx::query(
            "INSERT INTO websites (name, domain, root_path, port, ssl_enabled, nginx_config, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&req.name)
        .bind(&req.domain)
        .bind(&req.root_path)
        .bind(port)
        .bind(false)
        .bind(&nginx_config)
        .bind("stopped")
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        let id = result.last_insert_rowid();

        // 保存 Nginx 配置文件
        let config_path = self.nginx_config_dir.join(format!("{}.conf", req.domain));
        std::fs::create_dir_all(&self.nginx_config_dir)?;
        std::fs::write(&config_path, &nginx_config)?;

        Ok(Website {
            id,
            name: req.name.clone(),
            domain: req.domain.clone(),
            root_path: req.root_path.clone(),
            port,
            ssl_enabled: false,
            ssl_cert_path: None,
            ssl_key_path: None,
            nginx_config: Some(nginx_config),
            status: "stopped".to_string(),
            created_at: now,
        })
    }

    /// 更新网站
    pub async fn update(&self, id: i64, req: &UpdateWebsiteRequest) -> Result<Website> {
        let website = self.get_by_id(id).await?.context("Website not found")?;

        let name = req.name.as_ref().unwrap_or(&website.name);
        let domain = req.domain.as_ref().unwrap_or(&website.domain);
        let root_path = req.root_path.as_ref().unwrap_or(&website.root_path);
        let port = req.port.unwrap_or(website.port);

        let nginx_config = self.generate_nginx_config(name, domain, root_path, port, website.ssl_enabled)?;

        sqlx::query(
            "UPDATE websites SET name = ?, domain = ?, root_path = ?, port = ?, nginx_config = ? WHERE id = ?"
        )
        .bind(name)
        .bind(domain)
        .bind(root_path)
        .bind(port)
        .bind(&nginx_config)
        .bind(id)
        .execute(&self.db_pool)
        .await?;

        self.get_by_id(id).await?.context("Website not found")
    }

    /// 删除网站
    pub async fn delete(&self, id: i64) -> Result<()> {
        let website = self.get_by_id(id).await?.context("Website not found")?;

        // 删除 Nginx 配置文件
        let config_path = self.nginx_config_dir.join(format!("{}.conf", website.domain));
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
        }

        sqlx::query("DELETE FROM websites WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// 配置 SSL
    pub async fn configure_ssl(&self, id: i64, req: &SslConfigRequest) -> Result<Website> {
        let website = self.get_by_id(id).await?.context("Website not found")?;

        let nginx_config = self.generate_nginx_config(
            &website.name,
            &website.domain,
            &website.root_path,
            website.port,
            true,
        )?;

        sqlx::query(
            "UPDATE websites SET ssl_enabled = ?, ssl_cert_path = ?, ssl_key_path = ?, nginx_config = ? WHERE id = ?"
        )
        .bind(true)
        .bind(&req.cert_path)
        .bind(&req.key_path)
        .bind(&nginx_config)
        .bind(id)
        .execute(&self.db_pool)
        .await?;

        self.get_by_id(id).await?.context("Website not found")
    }

    /// 重载 Nginx
    pub async fn reload_nginx(&self) -> Result<()> {
        let output = std::process::Command::new("nginx")
            .arg("-s")
            .arg("reload")
            .output()
            .context("Failed to reload nginx")?;

        if !output.status.success() {
            anyhow::bail!("Nginx reload failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// 生成 Nginx 配置
    fn generate_nginx_config(
        &self,
        name: &str,
        domain: &str,
        root_path: &str,
        port: i32,
        ssl_enabled: bool,
    ) -> Result<String> {
        let config = if ssl_enabled {
            format!(r#"
server {{
    listen 80;
    server_name {domain};
    return 301 https://$server_name$request_uri;
}}

server {{
    listen 443 ssl http2;
    server_name {domain};

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    root {root_path};
    index index.html index.php;

    location / {{
        try_files $uri $uri/ =404;
    }}

    access_log /var/log/nginx/{domain}.access.log;
    error_log /var/log/nginx/{domain}.error.log;
}}
"#, domain = domain, root_path = root_path)
        } else {
            format!(r#"
server {{
    listen {port};
    server_name {domain};

    root {root_path};
    index index.html index.php;

    location / {{
        try_files $uri $uri/ =404;
    }}

    access_log /var/log/nginx/{domain}.access.log;
    error_log /var/log/nginx/{domain}.error.log;
}}
"#, port = port, domain = domain, root_path = root_path)
        };

        Ok(config)
    }
}
