//! 对齐 gin `tools/clearDeleteData`：列出含 `deleted_at` 的表、预览软删数据、物理清除。
use crate::error::{AppError, AppResult};
use crate::AppState;
use axum::extract::{Query, State};
use axum::routing::delete;
use axum::{Json, Router};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{Column, Row};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableQuery {
    pub table: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearBody {
    pub table: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResp {
    pub columns: Vec<String>,
    pub data: Vec<serde_json::Value>,
}

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/tables", axum::routing::get(list_tables))
        .route("/preview", axum::routing::get(preview_table))
        .route("/", delete(clear_deleted_rows_handler))
}

/// 含 `deleted_at` 的表名（与 sqlite_master 扫描一致）。
pub async fn table_names_with_deleted_at(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"SELECT name FROM sqlite_master
           WHERE type='table' AND sql IS NOT NULL AND sql LIKE '%deleted_at%'
           ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
}

/// 自动维护：只删除「已软删且删除时间早于 3 个月前」的行（手动 API 仍可按表全量清除）。
const AUTO_PURGE_SOFT_DELETE_OLDER_MONTHS: &str = "3";

pub async fn purge_all_soft_deleted(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let tables = table_names_with_deleted_at(pool).await?;
    let mut total = 0u64;
    for t in tables {
        let sql = format!(
            r#"DELETE FROM "{}" WHERE deleted_at IS NOT NULL
               AND datetime(deleted_at) < datetime('now', 'localtime', '-{} months')"#,
            t,
            AUTO_PURGE_SOFT_DELETE_OLDER_MONTHS
        );
        let r = sqlx::query(&sql).execute(pool).await?;
        total += r.rows_affected();
    }
    Ok(total)
}

pub async fn vacuum_sqlite(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("VACUUM").execute(pool).await?;
    Ok(())
}

fn validate_identifier(name: &str) -> AppResult<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::BadRequest("非法表名".into()));
    }
    Ok(())
}

async fn table_has_deleted_at(pool: &SqlitePool, table: &str) -> AppResult<bool> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT name FROM sqlite_master
           WHERE type='table' AND name = ? AND sql IS NOT NULL AND sql LIKE '%deleted_at%'"#,
    )
    .bind(table)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

pub async fn list_tables(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    let names = table_names_with_deleted_at(&state.pool)
        .await
        .map_err(AppError::from)?;
    Ok(Json(names))
}

async fn pragma_columns(pool: &SqlitePool, table: &str) -> Result<Vec<String>, sqlx::Error> {
    let sql = format!(r#"PRAGMA table_info("{}")"#, table);
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.try_get(1)?;
        out.push(name);
    }
    Ok(out)
}

fn decode_cell(row: &sqlx::sqlite::SqliteRow, i: usize) -> serde_json::Value {
    use serde_json::{json, Value};
    if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
        return match v {
            Some(n) => json!(n),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
        return match v {
            Some(f) => serde_json::Number::from_f64(f)
                .map(|n| json!(n))
                .unwrap_or(Value::Null),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(i) {
        return match v {
            Some(s) => json!(s),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(i) {
        return match v {
            Some(b) if !b.is_empty() => json!(format!("<binary {} bytes>", b.len())),
            _ => Value::Null,
        };
    }
    Value::Null
}

fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for i in 0..row.len() {
        let key = row.column(i).name().to_string();
        map.insert(key, decode_cell(row, i));
    }
    serde_json::Value::Object(map)
}

pub async fn preview_table(
    State(state): State<AppState>,
    Query(q): Query<TableQuery>,
) -> AppResult<Json<PreviewResp>> {
    let table = q.table.trim();
    validate_identifier(table)?;
    if !table_has_deleted_at(&state.pool, table).await? {
        return Err(AppError::BadRequest("表不存在或无 deleted_at 字段".into()));
    }

    let sql = format!(
        r#"SELECT * FROM "{}" WHERE deleted_at > '2020-01-01' LIMIT 100"#,
        table
    );
    let rows = sqlx::query(&sql)
        .fetch_all(&state.pool)
        .await
        .map_err(AppError::from)?;

    let columns = pragma_columns(&state.pool, table).await.map_err(AppError::from)?;
    let data: Vec<serde_json::Value> = rows.iter().map(|r| row_to_json(r)).collect();

    Ok(Json(PreviewResp { columns, data }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearedResp {
    pub deleted: u64,
}

async fn clear_deleted_rows_handler(
    State(state): State<AppState>,
    Json(body): Json<ClearBody>,
) -> AppResult<Json<ClearedResp>> {
    clear_deleted_rows(&state.pool, body.table.trim()).await
}

pub async fn clear_deleted_rows(pool: &SqlitePool, table: &str) -> AppResult<Json<ClearedResp>> {
    validate_identifier(table)?;
    if !table_has_deleted_at(pool, table).await? {
        return Err(AppError::BadRequest("表不存在或无 deleted_at 字段".into()));
    }

    let sql = format!(
        r#"DELETE FROM "{}" WHERE deleted_at > '2020-01-01'"#,
        table
    );
    let r = sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(Json(ClearedResp {
        deleted: r.rows_affected(),
    }))
}
