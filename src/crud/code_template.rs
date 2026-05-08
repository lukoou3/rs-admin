use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::sqlite::Sqlite;
use sqlx::{FromRow, QueryBuilder, SqlitePool};

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeTemplate {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub engine: Option<i64>,
    pub cate: Option<i64>,
    pub default_params: String,
    pub temp: String,
    #[serde(rename = "desc")]
    #[sqlx(rename = "note_desc")]
    pub desc: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeTemplateUpsert {
    pub name: String,
    pub engine: Option<i64>,
    pub cate: Option<i64>,
    #[serde(default)]
    pub default_params: String,
    #[serde(default)]
    pub temp: String,
    #[serde(default)]
    pub desc: String,
}

#[derive(Clone, Default)]
pub struct CodeTemplateListFilter {
    pub name: Option<String>,
    pub engine: Option<i64>,
    pub cate: Option<i64>,
    pub temp: Option<String>,
    pub desc: Option<String>,
    pub start_created_at: Option<String>,
    pub end_created_at: Option<String>,
}

fn push_filters<'a>(qb: &mut QueryBuilder<'a, Sqlite>, f: &'a CodeTemplateListFilter) {
    if let Some(n) = f.name.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND name LIKE ");
        qb.push_bind(format!("%{n}%"));
    }
    if let Some(engine) = f.engine {
        qb.push(" AND engine = ");
        qb.push_bind(engine);
    }
    if let Some(cate) = f.cate {
        qb.push(" AND cate = ");
        qb.push_bind(cate);
    }
    if let Some(t) = f.temp.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND temp LIKE ");
        qb.push_bind(format!("%{t}%"));
    }
    if let Some(d) = f.desc.as_ref().filter(|s| !s.is_empty()) {
        qb.push(r#" AND "desc" LIKE "#);
        qb.push_bind(format!("%{d}%"));
    }
    if let (Some(a), Some(b)) = (f.start_created_at.as_ref(), f.end_created_at.as_ref()) {
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
    f: &CodeTemplateListFilter,
) -> Result<(Vec<CodeTemplate>, i64), sqlx::Error> {
    let mut count_qb = QueryBuilder::<Sqlite>::new(
        "SELECT COUNT(*) FROM code_template WHERE deleted_at IS NULL",
    );
    push_filters(&mut count_qb, f);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Sqlite>::new(
        r#"SELECT id, created_at, updated_at, name, engine, cate,
                  default_params, temp, "desc" AS note_desc
           FROM code_template
           WHERE deleted_at IS NULL"#,
    );
    push_filters(&mut qb, f);
    qb.push(" ORDER BY id ASC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    let rows = qb.build_query_as::<CodeTemplate>().fetch_all(pool).await?;

    Ok((rows, total))
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<CodeTemplate>, sqlx::Error> {
    sqlx::query_as::<_, CodeTemplate>(
        r#"SELECT id, created_at, updated_at, name, engine, cate,
                  default_params, temp, "desc" AS note_desc
           FROM code_template
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &CodeTemplateUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO code_template
              (created_at, updated_at, name, engine, cate, default_params, temp, "desc")
           VALUES
              (datetime('now', 'localtime'), datetime('now', 'localtime'), ?, ?, ?, ?, ?, ?)
           RETURNING id"#,
    )
    .bind(&body.name)
    .bind(body.engine)
    .bind(body.cate)
    .bind(&body.default_params)
    .bind(&body.temp)
    .bind(&body.desc)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    body: &CodeTemplateUpsert,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE code_template
           SET updated_at = datetime('now', 'localtime'),
               name = ?, engine = ?, cate = ?, default_params = ?, temp = ?, "desc" = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&body.name)
    .bind(body.engine)
    .bind(body.cate)
    .bind(&body.default_params)
    .bind(&body.temp)
    .bind(&body.desc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE code_template
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
        r#"UPDATE code_template
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
