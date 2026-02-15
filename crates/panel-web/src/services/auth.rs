//! 认证服务

use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{User, UserInfo};

/// 认证服务
pub struct AuthService {
    db_pool: SqlitePool,
}

impl AuthService {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// 检查是否需要初始化
    pub async fn needs_setup(&self) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db_pool)
            .await?;

        Ok(count.0 == 0)
    }

    /// 创建初始管理员用户
    pub async fn setup(&self, username: &str, password: &str) -> Result<User> {
        if !self.needs_setup().await? {
            anyhow::bail!("System already set up");
        }

        let password_hash = hash(password, DEFAULT_COST)?;
        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?)"
        )
        .bind(username)
        .bind(&password_hash)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        let id = result.last_insert_rowid();

        Ok(User {
            id,
            username: username.to_string(),
            password_hash,
            created_at: now,
            last_login: None,
        })
    }

    /// 用户登录
    pub async fn login(&self, username: &str, password: &str) -> Result<User> {
        let user: User = sqlx::query_as(
            "SELECT id, username, password_hash, created_at, last_login FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.db_pool)
        .await?
        .context("User not found")?;

        if !verify(password, &user.password_hash)? {
            anyhow::bail!("Invalid password");
        }

        // 更新最后登录时间
        sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(user.id)
            .execute(&self.db_pool)
            .await?;

        Ok(user)
    }

    /// 根据 ID 获取用户
    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as(
            "SELECT id, username, password_hash, created_at, last_login FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(user)
    }

    /// 修改密码
    pub async fn change_password(&self, user_id: i64, old_password: &str, new_password: &str) -> Result<()> {
        let user = self.get_user_by_id(user_id).await?
            .context("User not found")?;

        if !verify(old_password, &user.password_hash)? {
            anyhow::bail!("Invalid old password");
        }

        let new_hash = hash(new_password, DEFAULT_COST)?;

        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(&new_hash)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}
