use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Datasource {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub alias: String,
    pub cate: Option<i64>,
    pub introduction: String,
    #[serde(rename = "sql")]
    #[sqlx(rename = "sql_body")]
    pub sql: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasourceUpsert {
    pub name: String,
    #[serde(default)]
    pub alias: String,
    pub cate: Option<i64>,
    #[serde(default)]
    pub introduction: String,
    #[serde(default)]
    pub sql: String,
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Datasource>, i64), sqlx::Error> {
    let (rows, total) = if let Some(k) = keyword.filter(|s| !s.is_empty()) {
        let pat = format!("%{k}%");
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sql_datasource WHERE deleted_at IS NULL AND name LIKE ?",
        )
        .bind(&pat)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, Datasource>(
            r#"SELECT id, created_at, updated_at, name, alias, cate, introduction, sql AS sql_body
               FROM sql_datasource
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
            sqlx::query_scalar("SELECT COUNT(*) FROM sql_datasource WHERE deleted_at IS NULL")
                .fetch_one(pool)
                .await?;

        let rows = sqlx::query_as::<_, Datasource>(
            r#"SELECT id, created_at, updated_at, name, alias, cate, introduction, sql AS sql_body
               FROM sql_datasource
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

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Datasource>, sqlx::Error> {
    sqlx::query_as::<_, Datasource>(
        r#"SELECT id, created_at, updated_at, name, alias, cate, introduction, sql AS sql_body
           FROM sql_datasource
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &DatasourceUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO sql_datasource (created_at, updated_at, name, alias, cate, introduction, sql)
           VALUES (datetime('now'), datetime('now'), ?, ?, ?, ?, ?)
           RETURNING id"#,
    )
    .bind(&body.name)
    .bind(&body.alias)
    .bind(body.cate)
    .bind(&body.introduction)
    .bind(&body.sql)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, body: &DatasourceUpsert) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sql_datasource
           SET updated_at = datetime('now'), name = ?, alias = ?, cate = ?, introduction = ?, sql = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&body.name)
    .bind(&body.alias)
    .bind(body.cate)
    .bind(&body.introduction)
    .bind(&body.sql)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sql_datasource
           SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
