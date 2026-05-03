//! 将已认证 API 请求元数据写入 `sys_operation_records`（不做 body/resp 记录；不记录 GET）。
use crate::auth::Claims;
use crate::crud::operation_record;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Method, Request};
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

const MAX_PATH: usize = 1024;
const MAX_AGENT: usize = 512;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max.saturating_sub(1) {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

fn client_ip<B>(req: &Request<B>) -> String {
    if let Some(xff) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = xff.split(',').next() {
            let s = first.trim();
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    if let Some(xr) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        let s = xr.trim();
        if !s.is_empty() {
            return s.to_string();
        }
    }
    String::new()
}

pub async fn middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // 与常见配置一致：不记录 GET，避免列表/轮询刷爆表。
    if *req.method() == Method::GET {
        return next.run(req).await;
    }

    let start = Instant::now();

    let method = req.method().as_str().to_string();
    let path = truncate(req.uri().path(), MAX_PATH);
    let ip = truncate(&client_ip(&req), 128);
    let agent = truncate(
        req.headers()
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        MAX_AGENT,
    );
    let user_id = req
        .extensions()
        .get::<Claims>()
        .and_then(|c| c.sub.parse().ok())
        .unwrap_or(0);

    let response = next.run(req).await;
    let status = response.status().as_u16() as i64;
    let latency_ns = start.elapsed().as_nanos().min(i64::MAX as u128) as i64;

    let pool = state.pool.clone();
    tokio::spawn(async move {
        if let Err(e) = operation_record::insert_metadata(
            &pool,
            user_id,
            &ip,
            &method,
            &path,
            status,
            latency_ns,
            &agent,
        )
        .await
        {
            tracing::warn!(error = %e, "写入操作记录失败");
        }
    });

    response
}
