use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(#[from] sqlx::Error),
    #[error("未找到")]
    NotFound,
    #[error("名称已存在")]
    Conflict,
    #[error("请求体无效")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
}

#[derive(Serialize)]
struct ErrBody {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Db(e) => {
                if let Some(db) = e.as_database_error() {
                    let msg = db.message();
                    if msg.contains("UNIQUE") {
                        return (StatusCode::CONFLICT, AppError::Conflict).into_response();
                    }
                }
                tracing::error!("{self}");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, self.to_string()),
            AppError::BadRequest(s) => (StatusCode::BAD_REQUEST, s.clone()),
            AppError::Unauthorized(s) => (StatusCode::UNAUTHORIZED, s.clone()),
        };

        let body = Json(ErrBody {
            code: status.as_u16(),
            message: msg,
        });
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
