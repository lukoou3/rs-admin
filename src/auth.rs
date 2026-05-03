//! JWT（HS256）与 Bearer 校验；Claims 挂在 request extensions。
use crate::error::AppError;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

pub const JWT_TTL_SECS: i64 = 7 * 24 * 3600;

pub fn sign_jwt(user_id: i64, username: &str, secret: &str) -> Result<(String, i64), AppError> {
    let now = chrono::Utc::now().timestamp();
    let exp = now + JWT_TTL_SECS;
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
        iat: now,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("jwt encode: {e}");
        AppError::Unauthorized("签发令牌失败".into())
    })?;
    Ok((token, exp))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized("令牌无效或已过期".into()))
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::trim));

    let Some(t) = token else {
        return AppError::Unauthorized("未登录或缺少令牌".into()).into_response();
    };

    match verify_jwt(t, state.jwt_secret.as_ref()) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}
