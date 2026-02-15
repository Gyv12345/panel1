//! Panel1 - Linux 服务器管理面板
//!
//! 主入口文件

use axum::Router;
use panel_web::create_router;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod services;
mod models;
mod middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "panel_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Panel1...");

    // 数据目录
    let data_dir = std::env::var("PANEL_DATA_DIR")
        .unwrap_or_else(|_| "/opt/panel/data".to_string());
    let data_path = PathBuf::from(&data_dir);

    // 确保数据目录存在
    std::fs::create_dir_all(&data_path)?;

    // 数据库路径
    let db_path = data_path.join("panel.db");
    let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

    tracing::info!("Database path: {}", db_path.display());

    // 连接数据库
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // 运行数据库迁移
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    tracing::info!("Database migrations completed");

    // 创建路由
    let app = create_router(db_pool.clone())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
