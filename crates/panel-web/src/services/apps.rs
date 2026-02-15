//! AI 应用管理服务

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::models::{App, AppTemplate, InstallAppRequest, get_app_templates};

/// AI 应用服务
pub struct AppService {
    db_pool: SqlitePool,
    apps_dir: PathBuf,
}

impl AppService {
    pub fn new(db_pool: SqlitePool, data_dir: &std::path::Path) -> Self {
        Self {
            db_pool,
            apps_dir: data_dir.join("apps"),
        }
    }

    /// 获取所有已安装应用
    pub async fn list(&self) -> Result<Vec<App>> {
        let apps = sqlx::query_as::<_, App>(
            "SELECT id, name, app_type, docker_compose_path, port, status, config, created_at FROM apps ORDER BY created_at DESC"
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(apps)
    }

    /// 获取应用模板列表
    pub fn get_templates() -> Vec<AppTemplate> {
        get_app_templates()
    }

    /// 获取应用模板
    pub fn get_template(app_type: &str) -> Option<AppTemplate> {
        get_app_templates().into_iter().find(|t| t.app_type == app_type)
    }

    /// 安装应用
    pub async fn install(&self, req: &InstallAppRequest) -> Result<App> {
        let template = Self::get_template(&req.app_type)
            .context("App template not found")?;

        let port = req.port.unwrap_or(template.default_port);
        let app_dir = self.apps_dir.join(&req.name);
        std::fs::create_dir_all(&app_dir)?;

        // 生成 docker-compose.yml
        let compose_content = self.generate_compose(&template, port, &req.environment_vars)?;
        let compose_path = app_dir.join("docker-compose.yml");
        std::fs::write(&compose_path, &compose_content)?;

        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO apps (name, app_type, docker_compose_path, port, status, config, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&req.name)
        .bind(&req.app_type)
        .bind(compose_path.to_string_lossy().to_string())
        .bind(port as i32)
        .bind("stopped")
        .bind(serde_json::to_string(&req.environment_vars)?)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        let id = result.last_insert_rowid();

        Ok(App {
            id,
            name: req.name.clone(),
            app_type: req.app_type.clone(),
            docker_compose_path: Some(compose_path.to_string_lossy().to_string()),
            port: Some(port as i32),
            status: "stopped".to_string(),
            config: Some(serde_json::to_string(&req.environment_vars)?),
            created_at: now,
        })
    }

    /// 启动应用
    pub async fn start(&self, id: i64) -> Result<()> {
        let app = self.get_by_id(id).await?.context("App not found")?;

        let compose_path = app.docker_compose_path.as_ref()
            .context("Docker compose path not set")?;

        let compose_path_buf = PathBuf::from(compose_path);
        let app_dir = compose_path_buf.parent()
            .context("Invalid compose path")?;

        let output = std::process::Command::new("docker")
            .args(["compose", "up", "-d"])
            .current_dir(app_dir)
            .output()
            .context("Failed to start app")?;

        if !output.status.success() {
            anyhow::bail!("Failed to start app: {}", String::from_utf8_lossy(&output.stderr));
        }

        sqlx::query("UPDATE apps SET status = ? WHERE id = ?")
            .bind("running")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// 停止应用
    pub async fn stop(&self, id: i64) -> Result<()> {
        let app = self.get_by_id(id).await?.context("App not found")?;

        let compose_path = app.docker_compose_path.as_ref()
            .context("Docker compose path not set")?;

        let compose_path_buf = PathBuf::from(compose_path);
        let app_dir = compose_path_buf.parent()
            .context("Invalid compose path")?;

        let output = std::process::Command::new("docker")
            .args(["compose", "down"])
            .current_dir(app_dir)
            .output()
            .context("Failed to stop app")?;

        if !output.status.success() {
            anyhow::bail!("Failed to stop app: {}", String::from_utf8_lossy(&output.stderr));
        }

        sqlx::query("UPDATE apps SET status = ? WHERE id = ?")
            .bind("stopped")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// 卸载应用
    pub async fn uninstall(&self, id: i64) -> Result<()> {
        let app = self.get_by_id(id).await?.context("App not found")?;

        // 先停止
        self.stop(id).await.ok();

        // 删除应用目录
        if let Some(compose_path) = &app.docker_compose_path {
            let compose_path_buf = PathBuf::from(compose_path);
            if let Some(dir) = compose_path_buf.parent() {
                std::fs::remove_dir_all(dir).ok();
            }
        }

        sqlx::query("DELETE FROM apps WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// 根据 ID 获取应用
    async fn get_by_id(&self, id: i64) -> Result<Option<App>> {
        let app = sqlx::query_as::<_, App>(
            "SELECT id, name, app_type, docker_compose_path, port, status, config, created_at FROM apps WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(app)
    }

    /// 生成 docker-compose.yml
    fn generate_compose(
        &self,
        template: &AppTemplate,
        port: u16,
        env_vars: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let mut environment = Vec::new();
        for env in &template.environment_vars {
            let value = env_vars.get(&env.name)
                .unwrap_or(&env.default_value);
            environment.push(format!("      - {}={}", env.name, value));
        }

        let mut volumes = Vec::new();
        for vol in &template.volumes {
            volumes.push(format!("      - {}:{}", vol.host_path, vol.container_path));
        }

        let compose = format!(r#"
version: '3.8'
services:
  {service_name}:
    image: {image}
    container_name: {service_name}
    restart: unless-stopped
    ports:
      - "{port}:{default_port}"
{environment}{volumes}
"#,
            service_name = template.app_type,
            image = template.docker_image,
            port = port,
            default_port = template.default_port,
            environment = if !environment.is_empty() {
                format!("    environment:\n{}\n", environment.join("\n"))
            } else {
                String::new()
            },
            volumes = if !volumes.is_empty() {
                format!("    volumes:\n{}\n", volumes.join("\n"))
            } else {
                String::new()
            }
        );

        Ok(compose)
    }
}
