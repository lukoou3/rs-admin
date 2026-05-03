use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

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
    sql: Option<&str>,
    desc: Option<&str>,
) -> Result<(Vec<QuerySql>, i64), sqlx::Error> {
    fn push_filters(
        qb: &mut QueryBuilder<'_, Sqlite>,
        keyword: Option<&str>,
        sql: Option<&str>,
        desc: Option<&str>,
    ) {
        if let Some(k) = keyword.filter(|s| !s.is_empty()) {
            qb.push(" AND name LIKE ");
            qb.push_bind(format!("%{k}%"));
        }
        if let Some(s) = sql.filter(|s| !s.is_empty()) {
            qb.push(" AND sql LIKE ");
            qb.push_bind(format!("%{s}%"));
        }
        if let Some(d) = desc.filter(|s| !s.is_empty()) {
            qb.push(r#" AND "desc" LIKE "#);
            qb.push_bind(format!("%{d}%"));
        }
    }

    let mut count_qb =
        QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM sql_querysql WHERE deleted_at IS NULL");
    push_filters(&mut count_qb, keyword, sql, desc);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let mut data_qb = QueryBuilder::<Sqlite>::new(
        r#"SELECT id, created_at, updated_at, name, cate, sql AS sql_body, "desc" AS note_desc
           FROM sql_querysql
           WHERE deleted_at IS NULL"#,
    );
    push_filters(&mut data_qb, keyword, sql, desc);
    data_qb.push(" ORDER BY id ASC LIMIT ");
    data_qb.push_bind(limit);
    data_qb.push(" OFFSET ");
    data_qb.push_bind(offset);
    let rows = data_qb.build_query_as::<QuerySql>().fetch_all(pool).await?;

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
           VALUES (datetime('now', 'localtime'), datetime('now', 'localtime'), ?, ?, ?, ?)
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
           SET updated_at = datetime('now', 'localtime'), name = ?, cate = ?, sql = ?, "desc" = ?
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
           SET deleted_at = datetime('now', 'localtime'), updated_at = datetime('now', 'localtime')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete_ids(pool: &SqlitePool, ids: &[i64]) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut qb = QueryBuilder::<Sqlite>::new(
        r#"UPDATE sql_querysql
           SET deleted_at = datetime('now', 'localtime'), updated_at = datetime('now', 'localtime')
           WHERE deleted_at IS NULL AND id IN ("#,
    );
    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(*id);
    }
    qb.push(")");
    let r = qb.build().execute(pool).await?;
    Ok(r.rows_affected())
}
