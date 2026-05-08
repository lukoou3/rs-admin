use crate::AppState;
use crate::auth;
use crate::crud::{
    code_template, datasource, dictionary, exec_script, operation_record, querysql, shellcode,
    sys_dictionary_admin, sys_users,
};
use crate::error::{AppError, AppResult};
use crate::exec_script_engine::{ExecScriptRunBody, RunInfoResponse};
use crate::html2md;
use crate::list_params::ListParams;
use crate::tools_clear_delete;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};
use sqlx::{Column, Row};
use std::time::{Duration, Instant};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageResp<T: Serialize> {
    list: Vec<T>,
    total: i64,
    page: u32,
    page_size: u32,
}

pub fn routes(state: AppState) -> Router {
    let api_protected = Router::new()
        .route("/dictionaries/{dict_type}", get(dictionary_by_type))
        .nest(
            "/shellcodes",
            Router::new()
                .route("/", get(shellcode_list).post(shellcode_create))
                .route("/delete-by-ids", post(shellcode_delete_by_ids))
                .route(
                    "/{id}",
                    get(shellcode_get)
                        .put(shellcode_update)
                        .delete(shellcode_delete),
                ),
        )
        .nest(
            "/sql-datasources",
            Router::new()
                .route("/", get(datasource_list).post(datasource_create))
                .route("/delete-by-ids", post(datasource_delete_by_ids))
                .route(
                    "/{id}",
                    get(datasource_get)
                        .put(datasource_update)
                        .delete(datasource_delete),
                ),
        )
        .nest(
            "/sql-queries",
            Router::new()
                .route("/", get(querysql_list).post(querysql_create))
                .route("/delete-by-ids", post(querysql_delete_by_ids))
                .route(
                    "/{id}",
                    get(querysql_get)
                        .put(querysql_update)
                        .delete(querysql_delete),
                ),
        )
        .nest(
            "/exec-scripts",
            Router::new()
                .route("/", get(exec_script_list).post(exec_script_create))
                .route("/run", post(exec_script_run))
                .route("/run-info", post(exec_script_run_info))
                .route("/is-running", post(exec_script_is_running))
                .route("/delete-by-ids", post(exec_script_delete_by_ids))
                .route(
                    "/{id}",
                    get(exec_script_get)
                        .put(exec_script_update)
                        .delete(exec_script_delete),
                ),
        )
        .nest(
            "/code-templates",
            Router::new()
                .route("/", get(code_template_list).post(code_template_create))
                .route("/render", post(code_template_render))
                .route("/delete-by-ids", post(code_template_delete_by_ids))
                .route(
                    "/{id}",
                    get(code_template_get)
                        .put(code_template_update)
                        .delete(code_template_delete),
                ),
        )
        .nest("/users", users_routes())
        .nest("/sys-dictionaries", sys_dictionaries_routes())
        .nest("/sys-dictionary-details", sys_dictionary_details_routes())
        .nest("/operation-records", operation_records_routes())
        .route("/query-platform/query", post(query_platform_query))
        .route("/tools/html2md", post(tools_html2md))
        .route("/tools/http/request", post(tools_http_request))
        .route("/tools/encode/hash/text", post(tools_encode_hash_text))
        .route("/tools/encode/hash/file", post(tools_encode_hash_file))
        .nest("/tools/clear-delete-data", tools_clear_delete::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::operation_log::middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    Router::new()
        .route("/health", get(health))
        .route("/api/auth/login", post(auth_login))
        .nest("/api", api_protected)
        .with_state(state)
}

fn users_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(users_list).post(users_create))
        .route("/{id}/password", put(users_reset_password))
        .route(
            "/{id}",
            get(users_get).put(users_update).delete(users_delete),
        )
}

fn sys_dictionaries_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(sys_dictionary_admin_list).post(sys_dictionary_admin_create),
        )
        .route("/delete-by-ids", post(sys_dictionary_admin_delete_by_ids))
        .route(
            "/{id}",
            get(sys_dictionary_admin_get)
                .put(sys_dictionary_admin_update)
                .delete(sys_dictionary_admin_delete),
        )
}

fn sys_dictionary_details_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(sys_dictionary_detail_list).post(sys_dictionary_detail_create),
        )
        .route(
            "/{id}",
            get(sys_dictionary_detail_get)
                .put(sys_dictionary_detail_update)
                .delete(sys_dictionary_detail_delete),
        )
}

fn operation_records_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(operation_record_list))
        .route("/delete-by-ids", post(operation_record_delete_by_ids))
        .route("/{id}", delete(operation_record_delete))
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginReq {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResp {
    token: String,
    expires_at: i64,
    user: LoginUserBrief,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginUserBrief {
    id: i64,
    user_name: String,
    nick_name: String,
}

async fn auth_login(
    State(state): State<AppState>,
    Json(body): Json<LoginReq>,
) -> AppResult<Json<LoginResp>> {
    let username = body.username.trim();
    if username.is_empty() || body.password.is_empty() {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }
    let row = sys_users::find_for_login(&state.pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".into()))?;
    if row.enable != 1 {
        return Err(AppError::Unauthorized("用户已被冻结".into()));
    }
    if !sys_users::verify_password(&body.password, &row.password) {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }
    let (token, exp) = auth::sign_jwt(row.id, username, state.jwt_secret.as_ref())?;
    Ok(Json(LoginResp {
        expires_at: exp,
        token,
        user: LoginUserBrief {
            id: row.id,
            user_name: row.username.clone(),
            nick_name: row.nick_name.clone(),
        },
    }))
}

async fn users_list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<Json<PageResp<sys_users::SysUserPublic>>> {
    let (offset, limit) = params.offset_limit();
    let (list, total) =
        sys_users::list(&state.pool, offset, limit, params.keyword.as_deref()).await?;
    Ok(Json(PageResp {
        list,
        total,
        page: params.page.max(1),
        page_size: params.page_size.clamp(1, 200),
    }))
}

async fn users_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<sys_users::SysUserPublic>> {
    let row = sys_users::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn users_create(
    State(state): State<AppState>,
    Json(body): Json<sys_users::SysUserCreate>,
) -> AppResult<Json<serde_json::Value>> {
    if body.user_name.trim().is_empty() {
        return Err(AppError::BadRequest("用户名不能为空".into()));
    }
    if body.password.is_empty() {
        return Err(AppError::BadRequest("密码不能为空".into()));
    }
    let id = sys_users::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn users_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<sys_users::SysUserUpdate>,
) -> AppResult<Json<serde_json::Value>> {
    let n = sys_users::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn users_delete(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    let n = sys_users::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn users_reset_password(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<sys_users::PasswordReset>,
) -> AppResult<Json<serde_json::Value>> {
    if body.new_password.is_empty() {
        return Err(AppError::BadRequest("新密码不能为空".into()));
    }
    let n = sys_users::update_password(&state.pool, id, &body.new_password).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DictListResp {
    list: Vec<dictionary::DictEntry>,
}

async fn tools_html2md(
    Json(body): Json<html2md::Html2mdReq>,
) -> AppResult<Json<html2md::Html2mdResp>> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(|| reqwest::Client::new());
    let resp = html2md::run_html2md(client, &body)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(resp))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpHeaderInput {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpToolReq {
    method: String,
    url: String,
    #[serde(default)]
    headers: Vec<HttpHeaderInput>,
    #[serde(default)]
    body: String,
    timeout_secs: Option<u64>,
    max_body_bytes: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpHeaderOut {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpToolResp {
    status: u16,
    status_text: String,
    elapsed_ms: u128,
    headers: Vec<HttpHeaderOut>,
    body: String,
    body_size: usize,
    truncated: bool,
}

async fn tools_http_request(Json(body): Json<HttpToolReq>) -> AppResult<Json<HttpToolResp>> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(reqwest::Client::new);

    let method = body
        .method
        .parse::<reqwest::Method>()
        .map_err(|e| AppError::BadRequest(format!("HTTP 方法无效: {e}")))?;
    let url = reqwest::Url::parse(body.url.trim())
        .map_err(|e| AppError::BadRequest(format!("URL 无效: {e}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest("只支持 http/https URL".into()));
    }

    let timeout = body.timeout_secs.unwrap_or(20).clamp(1, 300);
    let max_body_bytes = body
        .max_body_bytes
        .unwrap_or(2 * 1024 * 1024)
        .clamp(1024, 20 * 1024 * 1024);

    let mut req = client
        .request(method, url)
        .timeout(Duration::from_secs(timeout));
    for header in body.headers {
        let key = header.key.trim();
        if key.is_empty() {
            continue;
        }
        let name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| AppError::BadRequest(format!("请求头名称无效 {key}: {e}")))?;
        let value = reqwest::header::HeaderValue::from_str(&header.value)
            .map_err(|e| AppError::BadRequest(format!("请求头值无效 {key}: {e}")))?;
        req = req.header(name, value);
    }
    if !body.body.is_empty() {
        req = req.body(body.body);
    }

    let start = Instant::now();
    let mut resp = req
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("请求失败: {e}")))?;
    let status = resp.status();
    let headers = resp
        .headers()
        .iter()
        .map(|(key, value)| HttpHeaderOut {
            key: key.as_str().to_string(),
            value: value.to_str().unwrap_or("<非 UTF-8 响应头>").to_string(),
        })
        .collect::<Vec<_>>();

    let mut bytes = Vec::new();
    let mut body_size = 0usize;
    let mut truncated = false;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| AppError::BadRequest(format!("读取响应失败: {e}")))?
    {
        body_size = body_size.saturating_add(chunk.len());
        let remaining = max_body_bytes.saturating_sub(bytes.len());
        if remaining == 0 {
            truncated = true;
            continue;
        }
        if chunk.len() > remaining {
            bytes.extend_from_slice(&chunk[..remaining]);
            truncated = true;
        } else {
            bytes.extend_from_slice(&chunk);
        }
    }

    Ok(Json(HttpToolResp {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        elapsed_ms: start.elapsed().as_millis(),
        headers,
        body: String::from_utf8_lossy(&bytes).into_owned(),
        body_size,
        truncated,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HashTextBody {
    text: String,
    #[serde(default)]
    algorithms: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HashResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha512: Option<String>,
    size: u64,
    file_name: Option<String>,
}

#[derive(Clone, Copy)]
struct HashAlgorithms {
    md5: bool,
    sha1: bool,
    sha256: bool,
    sha512: bool,
}

impl Default for HashAlgorithms {
    fn default() -> Self {
        Self {
            md5: true,
            sha1: false,
            sha256: false,
            sha512: false,
        }
    }
}

impl HashAlgorithms {
    fn from_names(names: &[String]) -> AppResult<Self> {
        if names.is_empty() {
            return Ok(Self::default());
        }

        let mut out = Self {
            md5: false,
            sha1: false,
            sha256: false,
            sha512: false,
        };
        for name in names {
            match name.trim().to_ascii_lowercase().as_str() {
                "md5" => out.md5 = true,
                "sha1" | "sha-1" => out.sha1 = true,
                "sha256" | "sha-256" => out.sha256 = true,
                "sha512" | "sha-512" => out.sha512 = true,
                "" => {}
                other => {
                    return Err(AppError::BadRequest(format!("不支持的 Hash 算法: {other}")));
                }
            }
        }
        if out.md5 || out.sha1 || out.sha256 || out.sha512 {
            Ok(out)
        } else {
            Ok(Self::default())
        }
    }

    fn from_csv(value: Option<&str>) -> AppResult<Self> {
        let names = value
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        Self::from_names(&names)
    }
}

struct HashState {
    md5: Option<md5::Context>,
    sha1: Option<Sha1>,
    sha256: Option<Sha256>,
    sha512: Option<Sha512>,
    size: u64,
}

impl HashState {
    fn new(algorithms: HashAlgorithms) -> Self {
        Self {
            md5: algorithms.md5.then(md5::Context::new),
            sha1: algorithms.sha1.then(Sha1::new),
            sha256: algorithms.sha256.then(Sha256::new),
            sha512: algorithms.sha512.then(Sha512::new),
            size: 0,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        if let Some(md5) = &mut self.md5 {
            md5.consume(bytes);
        }
        if let Some(sha1) = &mut self.sha1 {
            sha1.update(bytes);
        }
        if let Some(sha256) = &mut self.sha256 {
            sha256.update(bytes);
        }
        if let Some(sha512) = &mut self.sha512 {
            sha512.update(bytes);
        }
        self.size += bytes.len() as u64;
    }

    fn finish(self, file_name: Option<String>) -> HashResp {
        HashResp {
            md5: self.md5.map(|ctx| format!("{:x}", ctx.finalize())),
            sha1: self.sha1.map(|ctx| format!("{:x}", ctx.finalize())),
            sha256: self.sha256.map(|ctx| format!("{:x}", ctx.finalize())),
            sha512: self.sha512.map(|ctx| format!("{:x}", ctx.finalize())),
            size: self.size,
            file_name,
        }
    }
}

fn hash_bytes(bytes: &[u8], file_name: Option<String>, algorithms: HashAlgorithms) -> HashResp {
    let mut state = HashState::new(algorithms);
    state.update(bytes);
    state.finish(file_name)
}

async fn tools_encode_hash_text(Json(body): Json<HashTextBody>) -> AppResult<Json<HashResp>> {
    let algorithms = HashAlgorithms::from_names(&body.algorithms)?;
    Ok(Json(hash_bytes(body.text.as_bytes(), None, algorithms)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HashFileQuery {
    file_name: Option<String>,
    algorithms: Option<String>,
}

async fn tools_encode_hash_file(
    Query(q): Query<HashFileQuery>,
    body: Body,
) -> AppResult<Json<HashResp>> {
    let mut stream = body.into_data_stream();
    let algorithms = HashAlgorithms::from_csv(q.algorithms.as_deref())?;
    let mut state = HashState::new(algorithms);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::BadRequest(format!("读取上传文件失败: {e}")))?;
        state.update(&chunk);
    }
    Ok(Json(state.finish(q.file_name)))
}

async fn dictionary_by_type(
    State(state): State<AppState>,
    Path(dict_type): Path<String>,
) -> AppResult<Json<DictListResp>> {
    let list = dictionary::list_by_type(&state.pool, &dict_type).await?;
    Ok(Json(DictListResp { list }))
}

fn default_page_one() -> u32 {
    1
}

fn default_ps_10() -> u32 {
    10
}

fn default_ps_20() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SysDictionaryAdminListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_10", rename = "pageSize")]
    page_size: u32,
    name: Option<String>,
    #[serde(rename = "type")]
    dict_type: Option<String>,
    desc: Option<String>,
    status: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SysDictionaryDetailListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_10", rename = "pageSize")]
    page_size: u32,
    #[serde(rename = "sysDictionaryID")]
    sys_dictionary_id: i64,
    label: Option<String>,
    value: Option<i64>,
    status: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperationRecordListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_10", rename = "pageSize")]
    page_size: u32,
    method: Option<String>,
    path: Option<String>,
    status: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ShellcodeListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_20")]
    page_size: u32,
    keyword: Option<String>,
    code: Option<String>,
    desc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DatasourceListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_20")]
    page_size: u32,
    keyword: Option<String>,
    sql: Option<String>,
    desc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuerySqlListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_20")]
    page_size: u32,
    keyword: Option<String>,
    sql: Option<String>,
    desc: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeTemplateListQuery {
    #[serde(default = "default_page_one")]
    page: u32,
    #[serde(default = "default_ps_20", rename = "pageSize")]
    page_size: u32,
    name: Option<String>,
    engine: Option<i64>,
    cate: Option<i64>,
    temp: Option<String>,
    desc: Option<String>,
    start_created_at: Option<String>,
    end_created_at: Option<String>,
}

async fn sys_dictionary_admin_list(
    State(state): State<AppState>,
    Query(q): Query<SysDictionaryAdminListQuery>,
) -> AppResult<Json<PageResp<sys_dictionary_admin::SysDictionaryHeader>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let (list, total) = sys_dictionary_admin::list_headers(
        &state.pool,
        offset,
        limit,
        q.name.as_deref(),
        q.dict_type.as_deref(),
        q.desc.as_deref(),
        q.status,
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn sys_dictionary_admin_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<sys_dictionary_admin::SysDictionaryHeader>> {
    let row = sys_dictionary_admin::get_header(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn sys_dictionary_admin_create(
    State(state): State<AppState>,
    Json(body): Json<sys_dictionary_admin::SysDictionaryCreate>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() || body.dict_type.trim().is_empty() {
        return Err(AppError::BadRequest("字典名称与类型不能为空".into()));
    }
    if sys_dictionary_admin::count_type_exists(&state.pool, body.dict_type.trim(), None).await? {
        return Err(AppError::Conflict);
    }
    let id = sys_dictionary_admin::create_header(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn sys_dictionary_admin_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<sys_dictionary_admin::SysDictionaryUpdate>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() || body.dict_type.trim().is_empty() {
        return Err(AppError::BadRequest("字典名称与类型不能为空".into()));
    }
    let existing = sys_dictionary_admin::get_header(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if existing.dict_type != body.dict_type.trim()
        && sys_dictionary_admin::count_type_exists(&state.pool, body.dict_type.trim(), Some(id))
            .await?
    {
        return Err(AppError::Conflict);
    }
    let n = sys_dictionary_admin::update_header(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn sys_dictionary_admin_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = sys_dictionary_admin::soft_delete_header_and_details(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_dictionary_admin_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    sys_dictionary_admin::soft_delete_headers_and_details(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_dictionary_detail_list(
    State(state): State<AppState>,
    Query(q): Query<SysDictionaryDetailListQuery>,
) -> AppResult<Json<PageResp<sys_dictionary_admin::SysDictionaryDetailRow>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    if q.sys_dictionary_id <= 0 {
        return Err(AppError::BadRequest("sysDictionaryID 无效".into()));
    }
    sys_dictionary_admin::get_header(&state.pool, q.sys_dictionary_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let (list, total) = sys_dictionary_admin::list_details_for_dict(
        &state.pool,
        q.sys_dictionary_id,
        offset,
        limit,
        q.label.as_deref(),
        q.value,
        q.status,
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn sys_dictionary_detail_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<sys_dictionary_admin::SysDictionaryDetailRow>> {
    let row = sys_dictionary_admin::get_detail(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn sys_dictionary_detail_create(
    State(state): State<AppState>,
    Json(body): Json<sys_dictionary_admin::SysDictionaryDetailUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.sys_dictionary_id <= 0 {
        return Err(AppError::BadRequest("sysDictionaryID 无效".into()));
    }
    sys_dictionary_admin::get_header(&state.pool, body.sys_dictionary_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let id = sys_dictionary_admin::create_detail(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn sys_dictionary_detail_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<sys_dictionary_admin::SysDictionaryDetailPatch>,
) -> AppResult<Json<serde_json::Value>> {
    let n = sys_dictionary_admin::update_detail(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn sys_dictionary_detail_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = sys_dictionary_admin::soft_delete_detail(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn operation_record_list(
    State(state): State<AppState>,
    Query(q): Query<OperationRecordListQuery>,
) -> AppResult<Json<PageResp<operation_record::OperationRecordOut>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let (list, total) = operation_record::list(
        &state.pool,
        offset,
        limit,
        q.method.as_deref(),
        q.path.as_deref(),
        q.status,
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn operation_record_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = operation_record::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn operation_record_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    operation_record::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn shellcode_list(
    State(state): State<AppState>,
    Query(q): Query<ShellcodeListQuery>,
) -> AppResult<Json<PageResp<shellcode::Shellcode>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let (list, total) = shellcode::list(
        &state.pool,
        offset,
        limit,
        q.keyword.as_deref(),
        q.code.as_deref(),
        q.desc.as_deref(),
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn shellcode_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<shellcode::Shellcode>> {
    let row = shellcode::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn shellcode_create(
    State(state): State<AppState>,
    Json(body): Json<shellcode::ShellcodeUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let id = shellcode::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn shellcode_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<shellcode::ShellcodeUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let n = shellcode::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn shellcode_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = shellcode::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn shellcode_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    shellcode::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn datasource_list(
    State(state): State<AppState>,
    Query(q): Query<DatasourceListQuery>,
) -> AppResult<Json<PageResp<datasource::Datasource>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let (list, total) = datasource::list(
        &state.pool,
        offset,
        limit,
        q.keyword.as_deref(),
        q.sql.as_deref(),
        q.desc.as_deref(),
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn datasource_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<datasource::Datasource>> {
    let row = datasource::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn datasource_create(
    State(state): State<AppState>,
    Json(body): Json<datasource::DatasourceUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let id = datasource::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn datasource_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<datasource::DatasourceUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let n = datasource::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn datasource_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = datasource::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn datasource_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    datasource::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn querysql_list(
    State(state): State<AppState>,
    Query(q): Query<QuerySqlListQuery>,
) -> AppResult<Json<PageResp<querysql::QuerySql>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let (list, total) = querysql::list(
        &state.pool,
        offset,
        limit,
        q.keyword.as_deref(),
        q.sql.as_deref(),
        q.desc.as_deref(),
    )
    .await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn querysql_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<querysql::QuerySql>> {
    let row = querysql::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn querysql_create(
    State(state): State<AppState>,
    Json(body): Json<querysql::QuerySqlUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let id = querysql::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn querysql_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<querysql::QuerySqlUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    let n = querysql::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn querysql_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = querysql::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn querysql_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    querysql::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryPlatformBody {
    sql: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryPlatformResp {
    columns: Vec<String>,
    data: Vec<serde_json::Value>,
}

fn normalize_query_sql(sql: &str) -> AppResult<String> {
    let mut s = sql.trim().to_string();
    while s.ends_with(';') {
        s.pop();
        s = s.trim_end().to_string();
    }
    if s.is_empty() {
        return Err(AppError::BadRequest("SQL 不能为空".into()));
    }
    if s.contains(';') {
        return Err(AppError::BadRequest("只允许执行单条查询 SQL".into()));
    }
    Ok(s)
}

fn contains_sql_word(sql: &str, word: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let target = word.as_bytes();
    if target.is_empty() || bytes.len() < target.len() {
        return false;
    }
    for i in 0..=bytes.len() - target.len() {
        if &bytes[i..i + target.len()] != target {
            continue;
        }
        let before = i
            .checked_sub(1)
            .and_then(|idx| bytes.get(idx))
            .copied()
            .unwrap_or(b' ');
        let after = bytes.get(i + target.len()).copied().unwrap_or(b' ');
        let before_word = before.is_ascii_alphanumeric() || before == b'_';
        let after_word = after.is_ascii_alphanumeric() || after == b'_';
        if !before_word && !after_word {
            return true;
        }
    }
    false
}

fn ensure_readonly_query(sql: &str) -> AppResult<()> {
    let lower = sql.trim_start().to_ascii_lowercase();
    let allowed_head = lower.starts_with("select ")
        || lower.starts_with("with ")
        || lower.starts_with("explain ")
        || lower.starts_with("pragma table_info")
        || lower.starts_with("pragma table_list")
        || lower.starts_with("pragma database_list")
        || lower.starts_with("pragma index_list")
        || lower.starts_with("pragma index_info");
    if !allowed_head {
        return Err(AppError::BadRequest(
            "只允许 SELECT / WITH / EXPLAIN / 部分只读 PRAGMA".into(),
        ));
    }
    for word in [
        "insert", "update", "delete", "drop", "alter", "create", "replace", "vacuum", "attach",
        "detach",
    ] {
        if contains_sql_word(sql, word) {
            return Err(AppError::BadRequest("只允许查询，禁止修改数据库".into()));
        }
    }
    Ok(())
}

fn sqlite_cell_to_json(row: &sqlx::sqlite::SqliteRow, i: usize) -> serde_json::Value {
    use serde_json::{Value, json};
    if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
        return v.map_or(Value::Null, |n| json!(n));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
        return match v {
            Some(f) => serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(i) {
        return v.map_or(Value::Null, |s| json!(s));
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(i) {
        return match v {
            Some(b) if !b.is_empty() => json!(format!("<binary {} bytes>", b.len())),
            _ => Value::Null,
        };
    }
    Value::Null
}

async fn query_platform_query(
    State(state): State<AppState>,
    Json(body): Json<QueryPlatformBody>,
) -> AppResult<Json<QueryPlatformResp>> {
    let sql = normalize_query_sql(&body.sql)?;
    ensure_readonly_query(&sql)?;

    let lower = sql.trim_start().to_ascii_lowercase();
    let executable_sql = if lower.starts_with("pragma ") || lower.starts_with("explain ") {
        sql
    } else {
        format!("SELECT * FROM ({sql}) LIMIT 100")
    };

    let rows = sqlx::query(&executable_sql).fetch_all(&state.pool).await?;
    let columns = rows
        .first()
        .map(|row| {
            row.columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let data = rows
        .iter()
        .map(|row| {
            let mut map = serde_json::Map::new();
            for i in 0..row.len() {
                let key = row.column(i).name().to_string();
                map.insert(key, sqlite_cell_to_json(row, i));
            }
            serde_json::Value::Object(map)
        })
        .collect();

    Ok(Json(QueryPlatformResp { columns, data }))
}

async fn code_template_list(
    State(state): State<AppState>,
    Query(q): Query<CodeTemplateListQuery>,
) -> AppResult<Json<PageResp<code_template::CodeTemplate>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let filter = code_template::CodeTemplateListFilter {
        name: q.name,
        engine: q.engine,
        cate: q.cate,
        temp: q.temp,
        desc: q.desc,
        start_created_at: q.start_created_at,
        end_created_at: q.end_created_at,
    };
    let (list, total) = code_template::list(&state.pool, offset, limit, &filter).await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn code_template_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<code_template::CodeTemplate>> {
    let row = code_template::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

fn validate_code_template_upsert(body: &code_template::CodeTemplateUpsert) -> AppResult<()> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    if body.engine.is_none() {
        return Err(AppError::BadRequest("engine 不能为空".into()));
    }
    if body.cate.is_none() {
        return Err(AppError::BadRequest("cate 不能为空".into()));
    }
    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&body.default_params)
        .map_err(|_| AppError::BadRequest("defaultParams 必须是合法 JSON 对象".into()))?;
    if body.temp.trim().is_empty() {
        return Err(AppError::BadRequest("temp 不能为空".into()));
    }
    Ok(())
}

async fn code_template_create(
    State(state): State<AppState>,
    Json(body): Json<code_template::CodeTemplateUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    validate_code_template_upsert(&body)?;
    let id = code_template::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn code_template_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<code_template::CodeTemplateUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    validate_code_template_upsert(&body)?;
    let n = code_template::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn code_template_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = code_template::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn code_template_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    code_template::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeTemplateRenderBody {
    engine: Option<i64>,
    params: String,
    temp: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CodeTemplateRenderResp {
    rst: String,
}

fn render_minijinja_template(temp: &str, params: &str) -> AppResult<String> {
    let data: serde_json::Value = serde_json::from_str(params)
        .map_err(|e| AppError::BadRequest(format!("params 必须是合法 JSON: {e}")))?;
    let mut env = minijinja::Environment::new();
    env.add_function("current_date", || {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });
    env.add_function("current_timestamp", || {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });
    let tmpl = env
        .template_from_str(temp)
        .map_err(|e| AppError::BadRequest(format!("模板解析失败: {e}")))?;
    tmpl.render(data)
        .map_err(|e| AppError::BadRequest(format!("模板渲染失败: {e}")))
}

static PYFMT_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\{([A-Za-z_][A-Za-z0-9_.-]*)\}").unwrap());

fn json_value_to_pyfmt_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => v.to_string(),
    }
}

fn lookup_json_path<'a>(
    mut cur: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur)
}

fn render_pyfmt_template(temp: &str, params: &str) -> AppResult<String> {
    let data: serde_json::Value = serde_json::from_str(params)
        .map_err(|e| AppError::BadRequest(format!("params 必须是合法 JSON: {e}")))?;
    if !data.is_object() {
        return Err(AppError::BadRequest("params 必须是 JSON 对象".into()));
    }
    Ok(PYFMT_RE
        .replace_all(temp, |caps: &regex::Captures<'_>| {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            lookup_json_path(&data, key)
                .map(json_value_to_pyfmt_string)
                .unwrap_or_else(|| caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string())
        })
        .into_owned())
}

async fn code_template_render(
    Json(body): Json<CodeTemplateRenderBody>,
) -> AppResult<Json<CodeTemplateRenderResp>> {
    let rst = if body.engine == Some(2) {
        render_pyfmt_template(&body.temp, &body.params)?
    } else {
        render_minijinja_template(&body.temp, &body.params)?
    };
    Ok(Json(CodeTemplateRenderResp { rst }))
}

fn default_page_es() -> u32 {
    1
}

fn default_page_size_es() -> u32 {
    10
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecScriptListQuery {
    #[serde(default = "default_page_es")]
    page: u32,
    #[serde(default = "default_page_size_es")]
    page_size: u32,
    name: Option<String>,
    cate: Option<i64>,
    interpreter: Option<String>,
    content: Option<String>,
    desc: Option<String>,
    start_created_at: Option<String>,
    end_created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IdsBody {
    ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunInfoBody {
    id: i64,
    #[serde(default)]
    encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdOnlyBody {
    id: i64,
}

fn validate_exec_script_upsert(body: &exec_script::ExecScriptUpsert) -> AppResult<()> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    if body.cate.is_none() {
        return Err(AppError::BadRequest("cate 不能为空".into()));
    }
    if body.interpreter.trim().is_empty() {
        return Err(AppError::BadRequest("interpreter 不能为空".into()));
    }
    if body.default_params.trim().is_empty() {
        return Err(AppError::BadRequest("defaultParams 不能为空".into()));
    }
    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&body.default_params)
        .map_err(|_| AppError::BadRequest("defaultParams 必须是合法 JSON 对象".into()))?;
    if body.content.trim().is_empty() {
        return Err(AppError::BadRequest("content 不能为空".into()));
    }
    Ok(())
}

async fn exec_script_list(
    State(state): State<AppState>,
    Query(q): Query<ExecScriptListQuery>,
) -> AppResult<Json<PageResp<exec_script::ExecScript>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;
    let filter = exec_script::ExecScriptListFilter {
        name: q.name,
        cate: q.cate,
        interpreter: q.interpreter,
        content: q.content,
        desc: q.desc,
        start_created_at: q.start_created_at,
        end_created_at: q.end_created_at,
    };
    let (list, total) = exec_script::list(&state.pool, offset, limit, &filter).await?;
    Ok(Json(PageResp {
        list,
        total,
        page,
        page_size,
    }))
}

async fn exec_script_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let row = exec_script::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::json!({ "rescript": row })))
}

async fn exec_script_create(
    State(state): State<AppState>,
    Json(body): Json<exec_script::ExecScriptUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    validate_exec_script_upsert(&body)?;
    let id = exec_script::create(&state.pool, &body).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn exec_script_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<exec_script::ExecScriptUpsert>,
) -> AppResult<Json<serde_json::Value>> {
    validate_exec_script_upsert(&body)?;
    let n = exec_script::update(&state.pool, id, &body).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn exec_script_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let n = exec_script::soft_delete(&state.pool, id).await?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn exec_script_delete_by_ids(
    State(state): State<AppState>,
    Json(body): Json<IdsBody>,
) -> AppResult<StatusCode> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不能为空".into()));
    }
    exec_script::soft_delete_ids(&state.pool, &body.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn exec_script_run(
    State(state): State<AppState>,
    Json(body): Json<ExecScriptRunBody>,
) -> AppResult<Json<serde_json::Value>> {
    state.exec_script_engine.try_run(body).await?;
    Ok(Json(serde_json::json!(1)))
}

async fn exec_script_run_info(
    State(state): State<AppState>,
    Json(body): Json<RunInfoBody>,
) -> AppResult<Json<RunInfoResponse>> {
    let enc = if body.encoding.trim().is_empty() {
        "utf-8"
    } else {
        body.encoding.trim()
    };
    let info = state
        .exec_script_engine
        .get_run_info(&state.pool, body.id, enc)
        .await?;
    Ok(Json(info))
}

async fn exec_script_is_running(
    State(state): State<AppState>,
    Json(body): Json<IdOnlyBody>,
) -> AppResult<Json<bool>> {
    let r = state.exec_script_engine.is_running(body.id).await;
    Ok(Json(r))
}
