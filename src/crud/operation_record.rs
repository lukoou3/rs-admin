//! `sys_operation_records` 列表与删除（与 gin-vue-admin 表一致）。
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
struct OperationJoinRow {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: i64,
    pub latency: i64,
    pub agent: String,
    #[sqlx(rename = "error_message")]
    pub error_message: String,
    pub body: String,
    pub resp: String,
    pub user_id: i64,
    pub username: Option<String>,
    #[sqlx(rename = "nick_name")]
    pub nick_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecordOut {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: i64,
    pub latency: i64,
    pub agent: String,
    pub error_message: String,
    pub body: String,
    pub resp: String,
    pub user_id: i64,
    pub user: Option<OperationUserOut>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationUserOut {
    pub user_name: String,
    pub nick_name: String,
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    method: Option<&str>,
    path: Option<&str>,
    status: Option<i64>,
) -> Result<(Vec<OperationRecordOut>, i64), sqlx::Error> {
    macro_rules! push_op_filters {
        ($qb:expr) => {{
            if let Some(m) = method.filter(|s| !s.is_empty()) {
                $qb.push(" AND o.method = ");
                $qb.push_bind(m);
            }
            if let Some(p) = path.filter(|s| !s.is_empty()) {
                $qb.push(" AND o.path LIKE ");
                $qb.push_bind(format!("%{p}%"));
            }
            if let Some(s) = status {
                $qb.push(" AND o.status = ");
                $qb.push_bind(s);
            }
        }};
    }

    let mut count_b = sqlx::QueryBuilder::new(
        r#"SELECT COUNT(*) FROM sys_operation_records o WHERE o.deleted_at IS NULL"#,
    );
    push_op_filters!(count_b);
    let total: i64 = count_b.build_query_scalar().fetch_one(pool).await?;

    let mut data_b = sqlx::QueryBuilder::new(
        r#"SELECT o.id, o.created_at, o.ip, o.method, o.path, o.status, o.latency, o.agent,
                  o.error_message, o.body, o.resp, o.user_id,
                  u.username AS username, u.nick_name AS nick_name
           FROM sys_operation_records o
           LEFT JOIN sys_users u ON o.user_id = u.id AND u.deleted_at IS NULL
           WHERE o.deleted_at IS NULL"#,
    );
    push_op_filters!(data_b);
    data_b.push(" ORDER BY o.id DESC LIMIT ");
    data_b.push_bind(limit);
    data_b.push(" OFFSET ");
    data_b.push_bind(offset);

    let rows = data_b
        .build_query_as::<OperationJoinRow>()
        .fetch_all(pool)
        .await?;

    let out = rows
        .into_iter()
        .map(|r| OperationRecordOut {
            id: r.id,
            created_at: r.created_at,
            ip: r.ip,
            method: r.method,
            path: r.path,
            status: r.status,
            latency: r.latency,
            agent: r.agent,
            error_message: r.error_message,
            body: r.body,
            resp: r.resp,
            user_id: r.user_id,
            user: match (r.username, r.nick_name) {
                (Some(un), Some(nn)) => Some(OperationUserOut {
                    user_name: un,
                    nick_name: nn,
                }),
                (Some(un), None) => Some(OperationUserOut {
                    user_name: un,
                    nick_name: String::new(),
                }),
                _ => None,
            },
        })
        .collect();

    Ok((out, total))
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sys_operation_records SET deleted_at = datetime('now'), updated_at = datetime('now')
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
    let mut qb = sqlx::QueryBuilder::new(
        r#"UPDATE sys_operation_records SET deleted_at = datetime('now'), updated_at = datetime('now')
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
