//! 系统监控 API 路由

use axum::Json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::ApiResponse;
use crate::services::system::SystemService;

lazy_static::lazy_static! {
    static ref SYSTEM_SERVICE: Arc<Mutex<SystemService>> =
        Arc::new(Mutex::new(SystemService::new()));
}

/// 系统信息响应
#[derive(serde::Serialize)]
pub struct SystemInfoResponse {
    pub system: panel_core::SystemInfo,
    pub cpu: panel_core::CpuInfo,
    pub memory: panel_core::MemoryInfo,
    pub disks: Vec<panel_core::DiskInfo>,
}

/// 获取系统信息
pub async fn get_info() -> Json<ApiResponse<SystemInfoResponse>> {
    let service = SYSTEM_SERVICE.lock().await;

    let response = SystemInfoResponse {
        system: service.get_system_info().await,
        cpu: service.get_cpu_info().await,
        memory: service.get_memory_info().await,
        disks: service.get_disk_info().await,
    };

    Json(ApiResponse::success(response))
}

/// 系统状态响应
#[derive(serde::Serialize)]
pub struct StatsResponse {
    pub cpu: panel_core::CpuInfo,
    pub memory: panel_core::MemoryInfo,
}

/// 获取实时系统状态
pub async fn get_stats() -> Json<ApiResponse<StatsResponse>> {
    let service = SYSTEM_SERVICE.lock().await;

    let response = StatsResponse {
        cpu: service.get_cpu_info().await,
        memory: service.get_memory_info().await,
    };

    Json(ApiResponse::success(response))
}

/// 获取进程列表
pub async fn get_processes() -> Json<ApiResponse<Vec<panel_core::ProcessInfo>>> {
    let service = SYSTEM_SERVICE.lock().await;
    let processes = service.get_processes().await;
    Json(ApiResponse::success(processes))
}

/// 获取网络信息
pub async fn get_network() -> Json<ApiResponse<Vec<panel_core::NetworkInterface>>> {
    let service = SYSTEM_SERVICE.lock().await;
    let interfaces = service.get_network_interfaces().await;
    Json(ApiResponse::success(interfaces))
}

/// 获取服务列表
pub async fn get_services() -> Json<ApiResponse<Vec<panel_core::ServiceInfo>>> {
    let service = SYSTEM_SERVICE.lock().await;
    match service.get_services() {
        Ok(services) => Json(ApiResponse::success(services)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
