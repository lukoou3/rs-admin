//! SQLite 定时维护：物理清除软删行、`VACUUM`（与 gin-vue-admin 启动 + 定时习惯对齐）。
use crate::config::MaintenanceConfig;
use crate::tools_clear_delete;
use sqlx::SqlitePool;

pub async fn run_startup_maintenance(pool: &SqlitePool, cfg: &MaintenanceConfig) {
    if !cfg.sqlite || !cfg.startup_vacuum {
        return;
    }
    match tools_clear_delete::vacuum_sqlite(pool).await {
        Ok(()) => tracing::info!("maintenance: 启动时 SQLite VACUUM 已完成"),
        Err(e) => tracing::warn!(error = %e, "maintenance: 启动时 VACUUM 失败"),
    }
}

pub fn spawn_scheduled_maintenance(pool: SqlitePool, cfg: MaintenanceConfig) {
    if !cfg.sqlite || !cfg.scheduled {
        return;
    }
    let interval = cfg.interval;
    tokio::spawn(async move {
        tracing::info!(
            secs = interval.as_secs(),
            "maintenance: 已启动定时任务（清除软删数据 + VACUUM），间隔秒数见上"
        );
        loop {
            tokio::time::sleep(interval).await;
            match tools_clear_delete::purge_all_soft_deleted(&pool).await {
                Ok(n) if n > 0 => {
                    tracing::info!(rows = n, "maintenance: 已物理清除软删行")
                }
                Ok(_) => tracing::debug!("maintenance: 无可清除的软删行"),
                Err(e) => tracing::warn!(error = %e, "maintenance: 清除软删失败"),
            }
            match tools_clear_delete::vacuum_sqlite(&pool).await {
                Ok(()) => tracing::info!("maintenance: 定时 VACUUM 已完成"),
                Err(e) => tracing::warn!(error = %e, "maintenance: 定时 VACUUM 失败"),
            }
        }
    });
}
