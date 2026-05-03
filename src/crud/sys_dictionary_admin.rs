//! `sys_dictionaries` / `sys_dictionary_details` 管理 CRUD（与 gin-vue-admin 表结构一致）。
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryHeader {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub dict_type: String,
    pub status: bool,
    #[serde(rename = "desc")]
    #[sqlx(rename = "desc")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryDetailRow {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub label: String,
    pub value: i64,
    pub status: bool,
    pub sort: i64,
    #[serde(rename = "sysDictionaryID")]
    #[sqlx(rename = "sys_dictionary_id")]
    pub sys_dictionary_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryCreate {
    pub name: String,
    #[serde(rename = "type")]
    pub dict_type: String,
    pub status: bool,
    #[serde(default)]
    #[serde(rename = "desc")]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryUpdate {
    pub name: String,
    #[serde(rename = "type")]
    pub dict_type: String,
    pub status: bool,
    #[serde(default)]
    #[serde(rename = "desc")]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryDetailUpsert {
    pub label: String,
    pub value: i64,
    pub status: bool,
    pub sort: i64,
    #[serde(rename = "sysDictionaryID")]
    pub sys_dictionary_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysDictionaryDetailPatch {
    pub label: String,
    pub value: i64,
    pub status: bool,
    pub sort: i64,
}

pub async fn count_type_exists(
    pool: &SqlitePool,
    dict_type: &str,
    exclude_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let r: i64 = if let Some(eid) = exclude_id {
        sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM sys_dictionaries
               WHERE deleted_at IS NULL AND type = ? AND id != ?"#,
        )
        .bind(dict_type)
        .bind(eid)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM sys_dictionaries WHERE deleted_at IS NULL AND type = ?"#,
        )
        .bind(dict_type)
        .fetch_one(pool)
        .await?
    };
    Ok(r > 0)
}

pub async fn list_headers(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    name: Option<&str>,
    dict_type: Option<&str>,
    desc: Option<&str>,
    status: Option<bool>,
) -> Result<(Vec<SysDictionaryHeader>, i64), sqlx::Error> {
    fn push_filters(
        qb: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
        name: Option<&str>,
        dict_type: Option<&str>,
        desc: Option<&str>,
        status: Option<bool>,
    ) {
        if let Some(n) = name.filter(|s| !s.is_empty()) {
            qb.push(" AND name LIKE ");
            qb.push_bind(format!("%{n}%"));
        }
        if let Some(t) = dict_type.filter(|s| !s.is_empty()) {
            qb.push(" AND type LIKE ");
            qb.push_bind(format!("%{t}%"));
        }
        if let Some(d) = desc.filter(|s| !s.is_empty()) {
            qb.push(" AND ");
            qb.push("\"desc\" LIKE ");
            qb.push_bind(format!("%{d}%"));
        }
        if let Some(st) = status {
            qb.push(" AND status = ");
            qb.push_bind(st);
        }
    }

    let mut count_b =
        sqlx::QueryBuilder::new("SELECT COUNT(*) FROM sys_dictionaries WHERE deleted_at IS NULL");
    push_filters(
        &mut count_b,
        name,
        dict_type,
        desc,
        status,
    );
    let total: i64 = count_b.build_query_scalar().fetch_one(pool).await?;

    let mut data_b = sqlx::QueryBuilder::new(
        r#"SELECT id, created_at, name, type, status, "desc" FROM sys_dictionaries WHERE deleted_at IS NULL"#,
    );
    push_filters(
        &mut data_b,
        name,
        dict_type,
        desc,
        status,
    );
    data_b.push(" ORDER BY id ASC LIMIT ");
    data_b.push_bind(limit);
    data_b.push(" OFFSET ");
    data_b.push_bind(offset);
    let rows = data_b.build_query_as::<SysDictionaryHeader>().fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn get_header(pool: &SqlitePool, id: i64) -> Result<Option<SysDictionaryHeader>, sqlx::Error> {
    sqlx::query_as::<_, SysDictionaryHeader>(
        r#"SELECT id, created_at, name, type, status, "desc" FROM sys_dictionaries WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_details_for_dict(
    pool: &SqlitePool,
    dict_id: i64,
    offset: i64,
    limit: i64,
    label: Option<&str>,
    value: Option<i64>,
    status: Option<bool>,
) -> Result<(Vec<SysDictionaryDetailRow>, i64), sqlx::Error> {
    fn push_detail_filters(
        qb: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
        dict_id: i64,
        label: Option<&str>,
        value: Option<i64>,
        status: Option<bool>,
    ) {
        qb.push(" AND sys_dictionary_id = ");
        qb.push_bind(dict_id);
        if let Some(l) = label.filter(|s| !s.is_empty()) {
            qb.push(" AND label LIKE ");
            qb.push_bind(format!("%{l}%"));
        }
        if let Some(v) = value {
            qb.push(" AND value = ");
            qb.push_bind(v);
        }
        if let Some(st) = status {
            qb.push(" AND status = ");
            qb.push_bind(st);
        }
    }

    let mut count_b = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) FROM sys_dictionary_details WHERE deleted_at IS NULL",
    );
    push_detail_filters(&mut count_b, dict_id, label, value, status);
    let total: i64 = count_b.build_query_scalar().fetch_one(pool).await?;

    let mut data_b = sqlx::QueryBuilder::new(
        r#"SELECT id, created_at, label, value, status, sort, sys_dictionary_id
           FROM sys_dictionary_details WHERE deleted_at IS NULL"#,
    );
    push_detail_filters(&mut data_b, dict_id, label, value, status);
    data_b.push(" ORDER BY sort ASC, id ASC LIMIT ");
    data_b.push_bind(limit);
    data_b.push(" OFFSET ");
    data_b.push_bind(offset);
    let rows = data_b
        .build_query_as::<SysDictionaryDetailRow>()
        .fetch_all(pool)
        .await?;
    Ok((rows, total))
}

pub async fn create_header(pool: &SqlitePool, body: &SysDictionaryCreate) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO sys_dictionaries (name, type, status, "desc", created_at, updated_at)
           VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
           RETURNING id"#,
    )
    .bind(body.name.trim())
    .bind(body.dict_type.trim())
    .bind(body.status)
    .bind(body.description.trim())
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update_header(
    pool: &SqlitePool,
    id: i64,
    body: &SysDictionaryUpdate,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sys_dictionaries SET name = ?, type = ?, status = ?, "desc" = ?, updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(body.name.trim())
    .bind(body.dict_type.trim())
    .bind(body.status)
    .bind(body.description.trim())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete_header_and_details(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let mut ex = pool.begin().await?;
    let r1 = sqlx::query(
        r#"UPDATE sys_dictionary_details SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE sys_dictionary_id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(&mut *ex)
    .await?;

    let r2 = sqlx::query(
        r#"UPDATE sys_dictionaries SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(&mut *ex)
    .await?;

    ex.commit().await?;
    Ok(r1.rows_affected() + r2.rows_affected())
}

pub async fn get_detail(pool: &SqlitePool, id: i64) -> Result<Option<SysDictionaryDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, SysDictionaryDetailRow>(
        r#"SELECT id, created_at, label, value, status, sort, sys_dictionary_id
           FROM sys_dictionary_details WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_detail(pool: &SqlitePool, body: &SysDictionaryDetailUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO sys_dictionary_details (label, value, status, sort, sys_dictionary_id, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
           RETURNING id"#,
    )
    .bind(body.label.trim())
    .bind(body.value)
    .bind(body.status)
    .bind(body.sort)
    .bind(body.sys_dictionary_id)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update_detail(
    pool: &SqlitePool,
    id: i64,
    body: &SysDictionaryDetailPatch,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sys_dictionary_details SET label = ?, value = ?, status = ?, sort = ?, updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(body.label.trim())
    .bind(body.value)
    .bind(body.status)
    .bind(body.sort)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete_detail(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sys_dictionary_details SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
