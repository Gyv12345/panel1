//! JWT 认证中间件

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT 密钥 (生产环境应从配置读取)
const JWT_SECRET: &[u8] = b"panel1_jwt_secret_key_change_in_production";

/// JWT Token 有效期 (24小时)
const TOKEN_EXPIRATION: u64 = 24 * 60 * 60;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,       // 用户 ID
    pub username: String,
    pub exp: usize,     // 过期时间
    pub iat: usize,     // 签发时间
}

/// 生成 JWT Token
pub fn generate_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: now + TOKEN_EXPIRATION as usize,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

/// 验证 JWT Token
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}

/// 认证中间件
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // 从 Header 获取 Token
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(
                    serde_json::json!({
                        "code": 401,
                        "message": "Missing or invalid Authorization header"
                    }).to_string()
                ))
                .unwrap();
        }
    };

    // 验证 Token
    match verify_token(token) {
        Ok(claims) => {
            // 将用户信息添加到请求扩展中
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(_) => {
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(
                    serde_json::json!({
                        "code": 401,
                        "message": "Invalid or expired token"
                    }).to_string()
                ))
                .unwrap()
        }
    }
}
