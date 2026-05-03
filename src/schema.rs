use sqlx::SqlitePool;

/// 与 gin-vue-admin `sys_operation_records` 对齐（仅本服务用到的列）。
pub async fn ensure_sys_operation_records_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sys_operation_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at TEXT,
            updated_at TEXT,
            deleted_at TEXT,
            user_id INTEGER NOT NULL DEFAULT 0,
            ip TEXT NOT NULL DEFAULT '',
            method TEXT NOT NULL DEFAULT '',
            path TEXT NOT NULL DEFAULT '',
            status INTEGER NOT NULL DEFAULT 0,
            latency INTEGER NOT NULL DEFAULT 0,
            agent TEXT NOT NULL DEFAULT '',
            error_message TEXT NOT NULL DEFAULT '',
            body TEXT NOT NULL DEFAULT '',
            resp TEXT NOT NULL DEFAULT ''
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ensure_exec_script_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exec_script (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at TEXT,
            updated_at TEXT,
            deleted_at TEXT,
            name TEXT NOT NULL DEFAULT '',
            cate INTEGER,
            interpreter TEXT NOT NULL DEFAULT '',
            encoding TEXT NOT NULL DEFAULT 'utf-8',
            default_params TEXT NOT NULL DEFAULT '{}',
            content TEXT NOT NULL DEFAULT '',
            "desc" TEXT NOT NULL DEFAULT '',
            last_exec_start_time TEXT,
            last_exec_end_time TEXT,
            last_exec_params TEXT,
            last_exec_info TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
