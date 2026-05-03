mod auth;
mod config;
mod crud;
mod error;
mod html2md;
mod exec_script_engine;
mod list_params;
mod operation_log;
mod router;
mod schema;
mod maintenance;
mod tools_clear_delete;

pub use config::CliArgs;
pub use error::AppError;

use anyhow::{Context, Result};
use axum::http::{header, Method};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub exec_script_engine: std::sync::Arc<exec_script_engine::ExecScriptEngine>,
    pub jwt_secret: std::sync::Arc<String>,
}

pub async fn run(args: CliArgs) -> Result<()> {
    let cfg = config::load(&args);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&cfg.database_url)
        .await
        .with_context(|| format!("打开数据库 {}", cfg.database_url))?;

    schema::ensure_exec_script_table(&pool)
        .await
        .with_context(|| "初始化 exec_script 表失败")?;
    schema::ensure_sys_operation_records_table(&pool)
        .await
        .with_context(|| "初始化 sys_operation_records 表失败")?;

    maintenance::run_startup_maintenance(&pool, &cfg.maintenance).await;
    maintenance::spawn_scheduled_maintenance(pool.clone(), cfg.maintenance);

    let state = AppState {
        pool,
        exec_script_engine: std::sync::Arc::new(exec_script_engine::ExecScriptEngine::new()),
        jwt_secret: std::sync::Arc::new(cfg.jwt_secret),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let dist = PathBuf::from(
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "web/dist".to_string()),
    );
    let index_html = dist.join("index.html");
    let mut app = router::routes(state);
    if dist.is_dir() && index_html.is_file() {
        tracing::info!("Serving static UI from {}", dist.display());
        let svc = ServeDir::new(&dist).fallback(ServeFile::new(index_html));
        app = app.fallback_service(svc);
    }

    let app = app.layer(cors).layer(TraceLayer::new_for_http());

    let addr: SocketAddr = cfg
        .listen
        .parse()
        .with_context(|| format!("无效监听地址 {}", cfg.listen))?;

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown");
}
