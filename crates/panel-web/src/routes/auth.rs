//! 认证 API 路由

use axum::{
    extract::Extension,
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::middleware::auth::{generate_token, Claims};
use crate::models::{
    ApiResponse, LoginRequest, LoginResponse, SetupRequest,
    ChangePasswordRequest, UserInfo,
};
use crate::services::auth::AuthService;

/// 系统初始化
pub async fn setup(
    Extension(db_pool): Extension<SqlitePool>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    let auth_service = AuthService::new(db_pool);

    match auth_service.setup(&req.username, &req.password).await {
        Ok(user) => {
            let token = generate_token(user.id, &user.username)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(ApiResponse::success(LoginResponse {
                token,
                user: UserInfo::from(user),
            })))
        }
        Err(e) => {
            tracing::error!("Setup failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 用户登录
pub async fn login(
    Extension(db_pool): Extension<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, Json<ApiResponse<()>>> {
    let auth_service = AuthService::new(db_pool);

    match auth_service.login(&req.username, &req.password).await {
        Ok(user) => {
            let token = generate_token(user.id, &user.username)
                .map_err(|e| {
                    tracing::error!("Token generation failed: {}", e);
                    Json(ApiResponse::error(500, "Failed to generate token"))
                })?;

            Ok(Json(ApiResponse::success(LoginResponse {
                token,
                user: UserInfo::from(user),
            })))
        }
        Err(e) => {
            tracing::warn!("Login failed: {}", e);
            Err(Json(ApiResponse::error(401, "Invalid username or password")))
        }
    }
}

/// 用户登出
pub async fn logout() -> Json<ApiResponse<()>> {
    Json(ApiResponse::success_with_message((), "Logged out successfully"))
}

/// 获取当前用户信息
pub async fn me(
    Extension(db_pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<UserInfo>>, Json<ApiResponse<()>>> {
    let auth_service = AuthService::new(db_pool);

    match auth_service.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => Ok(Json(ApiResponse::success(UserInfo::from(user)))),
        _ => Err(Json(ApiResponse::error(404, "User not found"))),
    }
}

/// 修改密码
pub async fn change_password(
    Extension(db_pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>, Json<ApiResponse<()>>> {
    let auth_service = AuthService::new(db_pool);

    match auth_service.change_password(claims.sub, &req.old_password, &req.new_password).await {
        Ok(()) => Ok(Json(ApiResponse::success_with_message((), "Password changed successfully"))),
        Err(e) => {
            tracing::warn!("Password change failed: {}", e);
            Err(Json(ApiResponse::error(400, &e.to_string())))
        }
    }
}
