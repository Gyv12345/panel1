//! API 路由模块

mod auth;
mod system;
mod files;
mod docker;
mod websites;
mod apps;

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::Extension,
    middleware,
};
use sqlx::SqlitePool;

use crate::middleware::auth::auth_middleware;

/// 创建应用路由
pub fn create_router(db_pool: SqlitePool) -> Router {
    // 公开路由 (无需认证)
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/setup", post(auth::setup))
        // 静态资源
        .fallback(crate::services::static_files::serve_static);

    // 需要认证的路由
    let protected_routes = Router::new()
        // 认证相关
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/password", put(auth::change_password))
        // 系统监控
        .route("/api/system/info", get(system::get_info))
        .route("/api/system/stats", get(system::get_stats))
        .route("/api/system/processes", get(system::get_processes))
        .route("/api/system/network", get(system::get_network))
        .route("/api/system/services", get(system::get_services))
        // 文件管理
        .route("/api/files", get(files::list))
        .route("/api/files/content", get(files::get_content))
        .route("/api/files", post(files::create))
        .route("/api/files", put(files::update))
        .route("/api/files", delete(files::delete_))
        .route("/api/files/upload", post(files::upload))
        .route("/api/files/download", get(files::download))
        // Docker 管理
        .route("/api/containers", get(docker::list_containers))
        .route("/api/containers", post(docker::create_container))
        .route("/api/containers/:id/start", post(docker::start_container))
        .route("/api/containers/:id/stop", post(docker::stop_container))
        .route("/api/containers/:id/restart", post(docker::restart_container))
        .route("/api/containers/:id", delete(docker::remove_container))
        .route("/api/containers/:id/logs", get(docker::container_logs))
        .route("/api/images", get(docker::list_images))
        .route("/api/images/pull", post(docker::pull_image))
        .route("/api/images/:id", delete(docker::remove_image))
        // 网站管理
        .route("/api/websites", get(websites::list))
        .route("/api/websites", post(websites::create))
        .route("/api/websites/:id", put(websites::update))
        .route("/api/websites/:id", delete(websites::delete_))
        .route("/api/websites/:id/ssl", post(websites::configure_ssl))
        .route("/api/websites/:id/reload", post(websites::reload_nginx))
        // AI 应用管理
        .route("/api/apps", get(apps::list))
        .route("/api/apps/templates", get(apps::templates))
        .route("/api/apps/install", post(apps::install))
        .route("/api/apps/:id/start", post(apps::start))
        .route("/api/apps/:id/stop", post(apps::stop))
        .route("/api/apps/:id", delete(apps::uninstall))
        .layer(middleware::from_fn(auth_middleware));

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(Extension(db_pool))
}
