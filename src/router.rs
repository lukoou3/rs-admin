use crate::auth;
use crate::crud::{
    datasource, dictionary, exec_script, operation_record, querysql, shellcode,
    sys_dictionary_admin, sys_users,
};
use crate::html2md;
use crate::tools_clear_delete;
use crate::error::{AppError, AppResult};
use crate::exec_script_engine::{ExecScriptRunBody, RunInfoResponse};
use crate::list_params::ListParams;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

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
        .nest("/users", users_routes())
        .nest("/sys-dictionaries", sys_dictionaries_routes())
        .nest("/sys-dictionary-details", sys_dictionary_details_routes())
        .nest("/operation-records", operation_records_routes())
        .route("/tools/html2md", post(tools_html2md))
        .nest("/tools/clear-delete-data", tools_clear_delete::routes())
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
            get(users_get)
                .put(users_update)
                .delete(users_delete),
        )
}

fn sys_dictionaries_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(sys_dictionary_admin_list).post(sys_dictionary_admin_create),
        )
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

async fn users_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
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
    Query(params): Query<ListParams>,
) -> AppResult<Json<PageResp<shellcode::Shellcode>>> {
    let (offset, limit) = params.offset_limit();
    let (list, total) = shellcode::list(&state.pool, offset, limit, params.keyword.as_deref()).await?;
    Ok(Json(PageResp {
        list,
        total,
        page: params.page.max(1),
        page_size: params.page_size.clamp(1, 200),
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

async fn datasource_list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<Json<PageResp<datasource::Datasource>>> {
    let (offset, limit) = params.offset_limit();
    let (list, total) =
        datasource::list(&state.pool, offset, limit, params.keyword.as_deref()).await?;
    Ok(Json(PageResp {
        list,
        total,
        page: params.page.max(1),
        page_size: params.page_size.clamp(1, 200),
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

async fn querysql_list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<Json<PageResp<querysql::QuerySql>>> {
    let (offset, limit) = params.offset_limit();
    let (list, total) = querysql::list(&state.pool, offset, limit, params.keyword.as_deref()).await?;
    Ok(Json(PageResp {
        list,
        total,
        page: params.page.max(1),
        page_size: params.page_size.clamp(1, 200),
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
