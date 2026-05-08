//! JWT（HS256）与 Bearer 校验；Claims 挂在 request extensions。
use crate::AppState;
use crate::error::AppError;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, Request, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

pub const JWT_TTL_SECS: i64 = 7 * 24 * 3600;
pub const JWT_REFRESH_BUFFER_SECS: i64 = 24 * 3600;

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

fn refresh_jwt_if_needed(claims: &Claims, secret: &str) -> Option<(String, i64)> {
    let now = chrono::Utc::now().timestamp();
    if claims.exp - now >= JWT_REFRESH_BUFFER_SECS {
        return None;
    }
    let user_id = claims.sub.parse::<i64>().ok()?;
    match sign_jwt(user_id, &claims.username, secret) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!(error = %e, "refresh jwt failed");
            None
        }
    }
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
            let refreshed = refresh_jwt_if_needed(&claims, state.jwt_secret.as_ref());
            req.extensions_mut().insert(claims);
            let mut resp = next.run(req).await;
            if let Some((new_token, new_expires_at)) = refreshed {
                if let Ok(v) = HeaderValue::from_str(&new_token) {
                    resp.headers_mut().insert("new-token", v);
                }
                if let Ok(v) = HeaderValue::from_str(&new_expires_at.to_string()) {
                    resp.headers_mut().insert("new-expires-at", v);
                }
            }
            resp
        }
        Err(e) => e.into_response(),
    }
}
