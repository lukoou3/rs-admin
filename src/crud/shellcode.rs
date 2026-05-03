use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shellcode {
    pub id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub code: String,
    #[serde(rename = "desc")]
    #[sqlx(rename = "note_desc")]
    pub desc: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellcodeUpsert {
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub desc: String,
}

pub async fn list(
    pool: &SqlitePool,
    offset: i64,
    limit: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Shellcode>, i64), sqlx::Error> {
    let (rows, total) = if let Some(k) = keyword.filter(|s| !s.is_empty()) {
        let pat = format!("%{k}%");
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM code_shellcode WHERE deleted_at IS NULL AND name LIKE ?",
        )
        .bind(&pat)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, Shellcode>(
            r#"SELECT id, created_at, updated_at, name, code, "desc" AS note_desc
               FROM code_shellcode
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
            sqlx::query_scalar("SELECT COUNT(*) FROM code_shellcode WHERE deleted_at IS NULL")
                .fetch_one(pool)
                .await?;

        let rows = sqlx::query_as::<_, Shellcode>(
            r#"SELECT id, created_at, updated_at, name, code, "desc" AS note_desc
               FROM code_shellcode
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

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Shellcode>, sqlx::Error> {
    sqlx::query_as::<_, Shellcode>(
        r#"SELECT id, created_at, updated_at, name, code, "desc" AS note_desc
           FROM code_shellcode
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, body: &ShellcodeUpsert) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO code_shellcode (created_at, updated_at, name, code, "desc")
           VALUES (datetime('now'), datetime('now'), ?, ?, ?)
           RETURNING id"#,
    )
    .bind(&body.name)
    .bind(&body.code)
    .bind(&body.desc)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, body: &ShellcodeUpsert) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE code_shellcode
           SET updated_at = datetime('now'), name = ?, code = ?, "desc" = ?
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(&body.name)
    .bind(&body.code)
    .bind(&body.desc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        r#"UPDATE code_shellcode
           SET deleted_at = datetime('now'), updated_at = datetime('now')
           WHERE id = ? AND deleted_at IS NULL"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
