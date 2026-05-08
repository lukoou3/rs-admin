use serde::Serialize;
use sqlx::FromRow;
use sqlx::SqlitePool;

/// 与 gin-vue-admin 前端 `getDictFunc(type)` 一致，读 `sys_dictionaries` + `sys_dictionary_details`
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntry {
    pub label: String,
    pub value: i64,
}

pub async fn list_by_type(
    pool: &SqlitePool,
    dict_type: &str,
) -> Result<Vec<DictEntry>, sqlx::Error> {
    sqlx::query_as::<_, DictEntry>(
        r#"SELECT d.label AS label, d.value AS value
           FROM sys_dictionary_details d
           INNER JOIN sys_dictionaries s
             ON s.id = d.sys_dictionary_id
            AND s.deleted_at IS NULL
           WHERE s.type = ?
             AND d.deleted_at IS NULL
           ORDER BY d.sort ASC, d.id ASC"#,
    )
    .bind(dict_type)
    .fetch_all(pool)
    .await
}
