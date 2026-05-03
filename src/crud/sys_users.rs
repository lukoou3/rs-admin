use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SysUserLoginRow {
    pub id: i64,
    pub username: String,
    pub password: String,
    #[sqlx(rename = "nick_name")]
    pub nick_name: String,
    pub enable: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SysUserPublic {
    pub id: i64,
    pub uuid: String,
    pub user_name: String,
    #[sqlx(rename = "nick_name")]
    pub nick_name: String,
    pub phone: String,
    pub email: String,
    pub enable: i64,
    #[sqlx(rename = "authority_id")]
    pub authority_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserCreate {
    pub user_name: String,
    pub password: String,
    #[serde(default)]
    pub nick_name: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserUpdate {
    #[serde(default)]
    pub nick_name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub enable: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordReset {
    pub new_password: String,
}

pub async fn find_for_login(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<SysUserLoginRow>, sqlx::Error> {
    sqlx::query_as::<_, SysUserLoginRow>(
        r#"SELECT id, username, password, nick_name, enable FROM sys_users
           WHERE deleted_at IS NULL AND username = ? LIMIT 1"#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    keyword: Option<&str>,
) -> Result<(Vec<SysUserPublic>, i64), sqlx::Error> {
    let (rows, total) = if let Some(k) = keyword.filter(|s| !s.is_empty()) {
        let pat = format!("%{k}%");
        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM sys_users
               WHERE deleted_at IS NULL AND (username LIKE ? OR nick_name LIKE ? OR phone LIKE ? OR email LIKE ?)"#,
        )
        .bind(&pat)
        .bind(&pat)
        .bind(&pat)
        .bind(&pat)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, SysUserPublic>(
            r#"SELECT id, uuid, username AS user_name, nick_name, phone, email, enable, authority_id, created_at, updated_at
               FROM sys_users
               WHERE deleted_at IS NULL AND (username LIKE ? OR nick_name LIKE ? OR phone LIKE ? OR email LIKE ?)
               ORDER BY id ASC LIMIT ? OFFSET ?"#,
        )
        .bind(&pat)
        .bind(&pat)
        .bind(&pat)
        .bind(&pat)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (rows, total)
    } else {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_users WHERE deleted_at IS NULL")
                .fetch_one(pool)
                .await?;

        let rows = sqlx::query_as::<_, SysUserPublic>(
            r#"SELECT id, uuid, username AS user_name, nick_name, phone, email, enable, authority_id, created_at, updated_at
               FROM sys_users
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

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<SysUserPublic>, sqlx::Error> {
    sqlx::query_as::<_, SysUserPublic>(
        r#"SELECT id, uuid, username AS user_name, nick_name, phone, email, enable, authority_id, created_at, updated_at
           FROM sys_users WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &SysUserCreate) -> Result<i64, sqlx::Error> {
    let pwd_hash = hash(body.password.as_str(), DEFAULT_COST).map_err(|e| {
        tracing::error!("bcrypt hash: {e}");
        sqlx::Error::Protocol("bcrypt hash failed".into())
    })?;
    let nick = if body.nick_name.trim().is_empty() {
        "系统用户".to_string()
    } else {
        body.nick_name.trim().to_string()
    };
    let uuid_str = Uuid::new_v4().simple().to_string();
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO sys_users (
            uuid, username, password, nick_name, side_mode, header_img, base_color, active_color,
            authority_id, phone, email, enable, created_at, updated_at
        ) VALUES (
            ?, ?, ?, ?, 'dark', '', '#fff', '#1890ff',
            888, ?, ?, 1,
            datetime('now', 'localtime'), datetime('now', 'localtime')
        )
        RETURNING id"#,
    )
    .bind(&uuid_str)
    .bind(body.user_name.trim())
    .bind(&pwd_hash)
    .bind(&nick)
    .bind(body.phone.trim())
    .bind(body.email.trim())
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    body: &SysUserUpdate,
) -> Result<u64, sqlx::Error> {
    let existing = get_by_id(pool, id).await?;
    let Some(cur) = existing else {
        return Ok(0);
    };

    let nick_name = body.nick_name.clone().unwrap_or(cur.nick_name);
    let phone = body.phone.clone().unwrap_or(cur.phone);
    let email = body.email.clone().unwrap_or(cur.email);
    let enable = body.enable.unwrap_or(cur.enable);

    let r = sqlx::query(
        r#"UPDATE sys_users SET nick_name = ?, phone = ?, email = ?, enable = ?, updated_at = datetime('now', 'localtime')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&nick_name)
    .bind(&phone)
    .bind(&email)
    .bind(enable)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn update_password(
    pool: &SqlitePool,
    id: i64,
    new_password: &str,
) -> Result<u64, sqlx::Error> {
    let pwd_hash = hash(new_password, DEFAULT_COST).map_err(|e| {
        tracing::error!("bcrypt hash: {e}");
        sqlx::Error::Protocol("bcrypt hash failed".into())
    })?;
    let r = sqlx::query(
        r#"UPDATE sys_users SET password = ?, updated_at = datetime('now', 'localtime')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&pwd_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE sys_users SET deleted_at = datetime('now', 'localtime'), updated_at = datetime('now', 'localtime')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub fn verify_password(plain: &str, hashed: &str) -> bool {
    verify(plain, hashed).unwrap_or(false)
}
