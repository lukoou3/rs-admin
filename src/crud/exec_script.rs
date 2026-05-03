use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::sqlite::Sqlite;
use sqlx::{FromRow, QueryBuilder, SqlitePool};

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecScript {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub cate: Option<i64>,
    pub interpreter: String,
    pub encoding: String,
    #[serde(rename = "defaultParams")]
    #[sqlx(rename = "default_params")]
    pub default_params: String,
    pub content: String,
    #[serde(rename = "desc")]
    #[sqlx(rename = "note_desc")]
    pub desc: String,
    pub last_exec_start_time: Option<NaiveDateTime>,
    pub last_exec_end_time: Option<NaiveDateTime>,
    #[serde(rename = "lastExecParams")]
    #[sqlx(rename = "last_exec_params")]
    pub last_exec_params: Option<String>,
    #[serde(rename = "lastExecInfo")]
    #[sqlx(rename = "last_exec_info")]
    pub last_exec_info: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecScriptUpsert {
    pub name: String,
    pub cate: Option<i64>,
    #[serde(default)]
    pub interpreter: String,
    #[serde(default)]
    pub encoding: String,
    #[serde(default)]
    pub default_params: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub desc: String,
}

#[derive(Clone, Default)]
pub struct ExecScriptListFilter {
    pub name: Option<String>,
    pub cate: Option<i64>,
    pub interpreter: Option<String>,
    pub desc: Option<String>,
    pub start_created_at: Option<String>,
    pub end_created_at: Option<String>,
}

fn push_exec_script_filters(qb: &mut QueryBuilder<'_, Sqlite>, f: &ExecScriptListFilter) {
    if let Some(n) = f.name.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND name LIKE ");
        qb.push_bind(format!("%{n}%"));
    }
    if let Some(c) = f.cate {
        qb.push(" AND cate = ");
        qb.push_bind(c);
    }
    if let Some(i) = f.interpreter.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND interpreter LIKE ");
        qb.push_bind(format!("%{i}%"));
    }
    if let Some(d) = f.desc.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND ");
        qb.push("\"desc\"");
        qb.push(" LIKE ");
        qb.push_bind(format!("%{d}%"));
    }
    if let (Some(a), Some(b)) = (f.start_created_at.clone(), f.end_created_at.clone()) {
        if !a.is_empty() && !b.is_empty() {
            qb.push(" AND created_at BETWEEN ");
            qb.push_bind(a);
            qb.push(" AND ");
            qb.push_bind(b);
        }
    }
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    f: &ExecScriptListFilter,
) -> Result<(Vec<ExecScript>, i64), sqlx::Error> {
    let mut count_qb = QueryBuilder::<Sqlite>::new(
        "SELECT COUNT(*) FROM exec_script WHERE deleted_at IS NULL ",
    );
    push_exec_script_filters(&mut count_qb, f);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let base = r#"SELECT id, created_at, updated_at, name, cate, interpreter, encoding,
              default_params, content, "desc" AS note_desc,
              last_exec_start_time, last_exec_end_time, last_exec_params, last_exec_info
       FROM exec_script WHERE deleted_at IS NULL "#;

    let mut qb = QueryBuilder::<Sqlite>::new(base);
    push_exec_script_filters(&mut qb, f);
    qb.push(" ORDER BY id ASC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb.build_query_as::<ExecScript>().fetch_all(pool).await?;

    Ok((rows, total))
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<ExecScript>, sqlx::Error> {
    sqlx::query_as::<_, ExecScript>(
        r#"SELECT id, created_at, updated_at, name, cate, interpreter, encoding,
                  default_params, content, "desc" AS note_desc,
                  last_exec_start_time, last_exec_end_time, last_exec_params, last_exec_info
           FROM exec_script WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &ExecScriptUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO exec_script (created_at, updated_at, name, cate, interpreter, encoding,
              default_params, content, "desc")
           VALUES (datetime('now'), datetime('now'), ?, ?, ?, ?, ?, ?, ?)
           RETURNING id"#,
    )
    .bind(&body.name)
    .bind(body.cate)
    .bind(&body.interpreter)
    .bind(&body.encoding)
    .bind(&body.default_params)
    .bind(&body.content)
    .bind(&body.desc)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, body: &ExecScriptUpsert) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE exec_script
           SET updated_at = datetime('now'), name = ?, cate = ?, interpreter = ?, encoding = ?,
               default_params = ?, content = ?, "desc" = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&body.name)
    .bind(body.cate)
    .bind(&body.interpreter)
    .bind(&body.encoding)
    .bind(&body.default_params)
    .bind(&body.content)
    .bind(&body.desc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE exec_script
           SET deleted_at = datetime('now'), updated_at = datetime('now')
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
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "UPDATE exec_script SET deleted_at = datetime('now'), updated_at = datetime('now') WHERE id IN ({placeholders}) AND deleted_at IS NULL"
    );
    let mut q = sqlx::query(&sql);
    for id in ids {
        q = q.bind(id);
    }
    let r = q.execute(pool).await?;
    Ok(r.rows_affected())
}

pub async fn update_last_exec(
    pool: &SqlitePool,
    id: i64,
    last_exec_start_time: NaiveDateTime,
    last_exec_end_time: NaiveDateTime,
    last_exec_params: &str,
    last_exec_info: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE exec_script SET updated_at = datetime('now'),
           last_exec_start_time = ?,
           last_exec_end_time = ?,
           last_exec_params = ?,
           last_exec_info = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(last_exec_start_time.to_string())
    .bind(last_exec_end_time.to_string())
    .bind(last_exec_params)
    .bind(last_exec_info)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
