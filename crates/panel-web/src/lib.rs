//! Panel Web - Web 服务和 API

pub mod routes;
pub mod services;
pub mod models;
pub mod middleware;

pub use routes::create_router;
