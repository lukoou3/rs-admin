use sqlx::SqlitePool;

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
