use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySql {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub cate: Option<i64>,
    #[serde(rename = "sql")]
    #[sqlx(rename = "sql_body")]
    pub sql: String,
    #[serde(rename = "desc")]
    #[sqlx(rename = "note_desc")]
    pub desc: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySqlUpsert {
    pub name: String,
    pub cate: Option<i64>,
    #[serde(default)]
    pub sql: String,
    #[serde(default)]
    pub desc: String,
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    keyword: Option<&str>,
) -> Result<(Vec<QuerySql>, i64), sqlx::Error> {
    let (rows, total) = if let Some(k) = keyword.filter(|s| !s.is_empty()) {
        let pat = format!("%{k}%");
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sql_querysql WHERE deleted_at IS NULL AND name LIKE ?",
        )
        .bind(&pat)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, QuerySql>(
            r#"SELECT id, created_at, updated_at, name, cate, sql AS sql_body, "desc" AS note_desc
               FROM sql_querysql
               WHERE deleted_at IS NULL AND name LIKE ?
               ORDER BY id ASC LIMIT ? OFFSET ?"#,
        )
        .bind(&pat)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (rows, total)
    } else {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sql_querysql WHERE deleted_at IS NULL")
                .fetch_one(pool)
                .await?;

        let rows = sqlx::query_as::<_, QuerySql>(
            r#"SELECT id, created_at, updated_at, name, cate, sql AS sql_body, "desc" AS note_desc
               FROM sql_querysql
               WHERE deleted_at IS NULL
               ORDER BY id ASC LIMIT ? OFFSET ?"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (rows, total)
    };

    Ok((rows, total))
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<QuerySql>, sqlx::Error> {
    sqlx::query_as::<_, QuerySql>(
        r#"SELECT id, created_at, updated_at, name, cate, sql AS sql_body, "desc" AS note_desc
           FROM sql_querysql
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &QuerySqlUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO sql_querysql (created_at, updated_at, name, cate, sql, "desc")
           VALUES (datetime('now'), datetime('now'), ?, ?, ?, ?)
           RETURNING id"#,
    )
    .bind(&body.name)
    .bind(body.cate)
    .bind(&body.sql)
    .bind(&body.desc)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, body: &QuerySqlUpsert) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sql_querysql
           SET updated_at = datetime('now'), name = ?, cate = ?, sql = ?, "desc" = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&body.name)
    .bind(body.cate)
    .bind(&body.sql)
    .bind(&body.desc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sql_querysql
           SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
