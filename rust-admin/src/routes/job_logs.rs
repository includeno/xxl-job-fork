use axum::{
    extract::{Form, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Local, LocalResult, Months, NaiveDateTime, TimeZone, Utc};
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use sea_orm::{
    query::*, ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::AuthUser;
use crate::entities::{job_group, job_log};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(page_list))
        .route("/{id}", get(detail))
        .route("/{id}/cat", get(log_content))
        .route("/{id}/kill", post(kill))
        .route("/clear", post(clear))
}

pub fn compat_router() -> Router<AppState> {
    Router::new()
        .route("/pageList", post(legacy_page_list))
        .route(
            "/logDetail",
            get(legacy_log_detail).post(legacy_log_detail_post),
        )
        .route("/logDetailCat", post(legacy_log_detail_cat))
        .route("/logKill", post(legacy_log_kill))
        .route("/clearLog", post(legacy_clear_log))
}

#[derive(Debug, Deserialize)]
struct PageParams {
    start: Option<u64>,
    length: Option<u64>,
    job_group: i32,
    job_id: Option<i32>,
    log_status: Option<i32>,
    filter_time: Option<String>,
}

#[derive(Debug, Serialize)]
struct PageResult<T> {
    records_total: u64,
    records_filtered: u64,
    data: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobLogDto {
    id: i64,
    job_group: i32,
    job_id: i32,
    executor_address: Option<String>,
    executor_handler: Option<String>,
    executor_param: Option<String>,
    trigger_time: Option<chrono::NaiveDateTime>,
    trigger_code: i32,
    trigger_msg: Option<String>,
    handle_time: Option<chrono::NaiveDateTime>,
    handle_code: i32,
    handle_msg: Option<String>,
    alarm_status: i8,
}

impl From<job_log::Model> for JobLogDto {
    fn from(value: job_log::Model) -> Self {
        Self {
            id: value.id,
            job_group: value.job_group,
            job_id: value.job_id,
            executor_address: value.executor_address,
            executor_handler: value.executor_handler,
            executor_param: value.executor_param,
            trigger_time: value.trigger_time,
            trigger_code: value.trigger_code,
            trigger_msg: value.trigger_msg,
            handle_time: value.handle_time,
            handle_code: value.handle_code,
            handle_msg: value.handle_msg,
            alarm_status: value.alarm_status,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyPageParams {
    start: Option<u64>,
    length: Option<u64>,
    job_group: i32,
    job_id: Option<i32>,
    log_status: Option<i32>,
    filter_time: Option<String>,
}

impl From<LegacyPageParams> for PageParams {
    fn from(value: LegacyPageParams) -> Self {
        Self {
            start: value.start,
            length: value.length,
            job_group: value.job_group,
            job_id: value.job_id,
            log_status: value.log_status,
            filter_time: value.filter_time,
        }
    }
}

#[derive(Debug, Serialize)]
struct LegacyReturn<T> {
    code: i32,
    msg: Option<String>,
    content: Option<T>,
}

impl<T> LegacyReturn<T> {
    fn success(content: Option<T>) -> Self {
        Self {
            code: 200,
            msg: None,
            content,
        }
    }

    fn success_with(content: T) -> Self {
        Self::success(Some(content))
    }

    fn failure(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: Some(msg.into()),
            content: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct LegacyLogIdParam {
    #[serde(alias = "logId")]
    id: i64,
}

#[derive(Debug, Deserialize)]
struct LegacyLogDetailCatParams {
    #[serde(rename = "logId")]
    log_id: i64,
    #[serde(rename = "fromLineNum")]
    from_line_num: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LegacyKillParams {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct LegacyClearRequest {
    #[serde(rename = "jobGroup")]
    job_group: i32,
    #[serde(rename = "jobId")]
    job_id: Option<i32>,
    #[serde(rename = "type")]
    clear_type: i32,
}

async fn page_list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<PageParams>,
) -> AppResult<Json<PageResult<JobLogDto>>> {
    let result = page_list_impl(&state, params).await?;
    Ok(Json(result))
}

async fn page_list_impl(state: &AppState, params: PageParams) -> AppResult<PageResult<JobLogDto>> {
    ensure_job_group(state, params.job_group).await?;

    let PageParams {
        start,
        length,
        job_group,
        job_id,
        log_status,
        filter_time,
    } = params;

    let mut query = job_log::Entity::find().filter(job_log::Column::JobGroup.eq(job_group));

    if let Some(job_id) = job_id {
        if job_id > 0 {
            query = query.filter(job_log::Column::JobId.eq(job_id));
        }
    }

    if let Some(status) = log_status {
        if status == 1 {
            query = query.filter(job_log::Column::HandleCode.eq(200));
        } else if status == 2 {
            query = query.filter(job_log::Column::HandleCode.ne(200));
        }
    }

    if let Some(range) = filter_time.as_ref() {
        if let Some((start_time, end_time)) = parse_time_range(range) {
            query = query
                .filter(job_log::Column::TriggerTime.gte(start_time))
                .filter(job_log::Column::TriggerTime.lte(end_time));
        }
    }

    let offset = start.unwrap_or(0);
    let limit = length.unwrap_or(10);

    let total = query.clone().count(state.db()).await? as u64;
    let data = query
        .order_by_desc(job_log::Column::TriggerTime)
        .offset(offset)
        .limit(limit)
        .all(state.db())
        .await?
        .into_iter()
        .map(JobLogDto::from)
        .collect();

    Ok(PageResult {
        records_total: total,
        records_filtered: total,
        data,
    })
}

async fn detail(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<JobLogDto>> {
    let log = detail_impl(&state, id).await?;
    Ok(Json(log))
}

async fn detail_impl(state: &AppState, id: i64) -> AppResult<JobLogDto> {
    let log = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;
    Ok(JobLogDto::from(log))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogContentDto {
    from_line_num: i64,
    to_line_num: i64,
    end: bool,
    log_content: String,
}

async fn log_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
    Query(params): Query<LogCatParams>,
) -> AppResult<Json<LogContentDto>> {
    let from = params.from_line_num.unwrap_or(1).max(1);
    let content = log_content_impl(&state, id, from).await?;
    Ok(Json(content))
}

async fn log_content_impl(state: &AppState, id: i64, from: i64) -> AppResult<LogContentDto> {
    let log = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;
    let summary_content = format!(
        "调度日志:\n{}\n执行日志:\n{}",
        log.trigger_msg.clone().unwrap_or_default(),
        log.handle_msg.clone().unwrap_or_default()
    );
    let from = from.max(1);

    if log.trigger_code != 200 && log.handle_code == 0 {
        return Ok(build_summary(
            from,
            &summary_content,
            Some("任务调度失败，执行日志不可用".into()),
        ));
    }

    let executor_address = match log
        .executor_address
        .as_deref()
        .map(str::trim)
        .filter(|addr| !addr.is_empty())
    {
        Some(address) => address,
        None => {
            return Ok(build_summary(
                from,
                &summary_content,
                Some("执行器地址缺失，返回摘要日志".into()),
            ))
        }
    };

    let trigger_time = match log.trigger_time {
        Some(time) => time,
        None => {
            return Ok(build_summary(
                from,
                &summary_content,
                Some("触发时间缺失，返回摘要日志".into()),
            ))
        }
    };

    let trigger_timestamp = match Local.from_local_datetime(&trigger_time) {
        LocalResult::Single(value) => value.timestamp_millis(),
        _ => Utc.from_utc_datetime(&trigger_time).timestamp_millis(),
    };

    #[derive(Serialize)]
    struct ExecutorLogRequest {
        #[serde(rename = "logId")]
        log_id: i64,
        #[serde(rename = "logDateTim")]
        log_date_tim: i64,
        #[serde(rename = "fromLineNum")]
        from_line_num: i64,
    }

    let mut url = executor_address.to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("log");

    let mut request = state
        .http_client()
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .json(&ExecutorLogRequest {
            log_id: log.id,
            log_date_tim: trigger_timestamp,
            from_line_num: from,
        })
        .timeout(state.settings().executor.timeout());

    if let Some(token) = state.settings().executor.access_token() {
        let header_value = HeaderValue::from_str(token)
            .map_err(|err| AppError::BadRequest(format!("访问令牌格式非法: {err}")))?;
        request = request.header(
            HeaderName::from_static("xxl-job-access-token"),
            header_value,
        );
    }

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(err) => {
            let message = if err.is_timeout() {
                format!("请求执行器超时: {err}")
            } else if err.is_connect() {
                format!("无法连接到执行器: {err}")
            } else {
                format!("调用执行器失败: {err}")
            };
            return Ok(build_summary(from, &summary_content, Some(message)));
        }
    };

    if !response.status().is_success() {
        let message = format!("执行器返回状态码 {}", response.status());
        return Ok(build_summary(from, &summary_content, Some(message)));
    }

    #[derive(Deserialize)]
    struct ExecutorLogResponse {
        code: i32,
        msg: Option<String>,
        content: Option<ExecutorLogContent>,
    }

    #[derive(Deserialize)]
    struct ExecutorLogContent {
        #[serde(rename = "fromLineNum")]
        from_line_num: i64,
        #[serde(rename = "toLineNum")]
        to_line_num: i64,
        #[serde(rename = "logContent")]
        log_content: String,
        #[serde(rename = "isEnd")]
        is_end: Option<bool>,
    }

    let payload = match response.json::<ExecutorLogResponse>().await {
        Ok(body) => body,
        Err(err) => {
            let message = format!("解析执行器日志响应失败: {err}");
            return Ok(build_summary(from, &summary_content, Some(message)));
        }
    };

    if payload.code != 200 {
        let message = payload.msg.unwrap_or_else(|| "执行器返回失败".into());
        return Ok(build_summary(from, &summary_content, Some(message)));
    }

    if let Some(content) = payload.content {
        return Ok(LogContentDto {
            from_line_num: content.from_line_num,
            to_line_num: content.to_line_num,
            end: content.is_end.unwrap_or(false),
            log_content: content.log_content,
        });
    }

    Ok(build_summary(
        from,
        &summary_content,
        Some("执行器未返回日志内容".into()),
    ))
}

fn build_summary(from: i64, summary: &str, reason: Option<String>) -> LogContentDto {
    let mut content = summary.to_string();
    if let Some(extra) = reason.and_then(|msg| {
        let trimmed = msg.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }) {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("\n提示: ");
        content.push_str(&extra);
    }
    let total_lines = content.lines().count() as i64;
    LogContentDto {
        from_line_num: from,
        to_line_num: total_lines,
        end: true,
        log_content: content,
    }
}

#[derive(Debug, Deserialize)]
struct LogCatParams {
    #[serde(rename = "fromLineNum")]
    from_line_num: Option<i64>,
}

async fn kill(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let message = kill_impl(&state, &user, id).await?;
    Ok(Json(json!({ "message": message })))
}

async fn kill_impl(state: &AppState, user: &AuthUser, id: i64) -> AppResult<String> {
    user.require_admin()?;
    let mut model = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;

    if model.handle_code == 200 {
        return Err(AppError::BadRequest("任务已完成，无需终止".into()));
    }

    model.handle_code = 500;
    model.handle_time = Some(Local::now().naive_local());
    let note = format!("操作人 {} 强制终止任务", user.username);
    model.handle_msg = Some(match model.handle_msg {
        Some(existing) => format!("{}\n{}", existing, &note),
        None => note,
    });

    let active: job_log::ActiveModel = model.into();
    active.update(state.db()).await?;

    Ok("已标记为终止".to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClearRequest {
    job_group: i32,
    job_id: Option<i32>,
    clear_before_days: Option<i64>,
    #[serde(default)]
    clear_before_time: Option<NaiveDateTime>,
    clear_before_rows: Option<i64>,
}

async fn clear(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<ClearRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = clear_impl(&state, &user, payload).await?;
    Ok(Json(json!({ "deleted": deleted })))
}

async fn clear_impl(state: &AppState, user: &AuthUser, payload: ClearRequest) -> AppResult<u64> {
    user.require_admin()?;
    ensure_job_group(state, payload.job_group).await?;

    let job_group = payload.job_group;
    let job_id = payload.job_id;

    let before_time = if let Some(time) = payload.clear_before_time {
        Some(time)
    } else if let Some(days) = payload.clear_before_days {
        if days > 0 {
            Some((Local::now() - Duration::days(days)).naive_local())
        } else {
            None
        }
    } else {
        None
    };

    let keep_recent = payload.clear_before_rows.filter(|value| *value > 0);

    delete_logs(state, job_group, job_id, before_time, keep_recent).await
}

async fn delete_logs(
    state: &AppState,
    job_group: i32,
    job_id: Option<i32>,
    before_time: Option<NaiveDateTime>,
    keep_recent: Option<i64>,
) -> AppResult<u64> {
    let mut base_condition = Condition::all().add(job_log::Column::JobGroup.eq(job_group));
    if let Some(id) = job_id {
        if id > 0 {
            base_condition = base_condition.add(job_log::Column::JobId.eq(id));
        }
    }

    let mut filter_condition = base_condition.clone();
    if let Some(time) = before_time {
        filter_condition = filter_condition.add(job_log::Column::TriggerTime.lte(time));
    }

    if keep_recent.unwrap_or(0) <= 0 {
        let result = job_log::Entity::delete_many()
            .filter(filter_condition)
            .exec(state.db())
            .await?;
        return Ok(result.rows_affected);
    }

    let limit = keep_recent.unwrap();
    let mut keep_condition = Condition::all().add(job_log::Column::JobGroup.eq(job_group));
    if let Some(id) = job_id {
        if id > 0 {
            keep_condition = keep_condition.add(job_log::Column::JobId.eq(id));
        }
    }

    let keep_ids: Vec<i64> = job_log::Entity::find()
        .select_only()
        .column(job_log::Column::Id)
        .filter(keep_condition)
        .order_by_desc(job_log::Column::TriggerTime)
        .limit(limit as u64)
        .into_tuple::<i64>()
        .all(state.db())
        .await?;

    if keep_ids.is_empty() {
        let result = job_log::Entity::delete_many()
            .filter(filter_condition)
            .exec(state.db())
            .await?;
        return Ok(result.rows_affected);
    }

    let mut total_deleted = 0u64;
    loop {
        let mut candidates = job_log::Entity::find()
            .select_only()
            .column(job_log::Column::Id)
            .filter(filter_condition.clone())
            .order_by_asc(job_log::Column::Id)
            .limit(1000);

        candidates = candidates.filter(job_log::Column::Id.is_not_in(keep_ids.clone()));

        let ids = candidates.into_tuple::<i64>().all(state.db()).await?;
        if ids.is_empty() {
            break;
        }

        let result = job_log::Entity::delete_many()
            .filter(job_log::Column::Id.is_in(ids.clone()))
            .exec(state.db())
            .await?;
        total_deleted += result.rows_affected;

        if ids.len() < 1000 {
            break;
        }
    }

    Ok(total_deleted)
}

fn convert_legacy_clear_request(params: LegacyClearRequest) -> Result<ClearRequest, String> {
    let LegacyClearRequest {
        job_group,
        job_id,
        clear_type,
    } = params;

    let normalized_job_id = job_id.filter(|id| *id > 0);

    let mut clear_before_time = None;
    let mut clear_before_rows = None;

    let now = Local::now();

    fn months_ago(now: chrono::DateTime<Local>, months: u32) -> Option<NaiveDateTime> {
        now.checked_sub_months(Months::new(months))
            .map(|dt| dt.naive_local())
    }

    match clear_type {
        1 => clear_before_time = months_ago(now, 1),
        2 => clear_before_time = months_ago(now, 3),
        3 => clear_before_time = months_ago(now, 6),
        4 => clear_before_time = months_ago(now, 12),
        5 => clear_before_rows = Some(1_000),
        6 => clear_before_rows = Some(10_000),
        7 => clear_before_rows = Some(30_000),
        8 => clear_before_rows = Some(100_000),
        9 => {
            clear_before_time = None;
            clear_before_rows = None;
        }
        _ => return Err("清理类型无效".into()),
    }

    Ok(ClearRequest {
        job_group,
        job_id: normalized_job_id,
        clear_before_days: None,
        clear_before_time,
        clear_before_rows,
    })
}

async fn legacy_page_list(
    State(state): State<AppState>,
    _user: AuthUser,
    Form(params): Form<LegacyPageParams>,
) -> AppResult<Json<PageResult<JobLogDto>>> {
    let result = page_list_impl(&state, params.into()).await?;
    Ok(Json(result))
}

async fn legacy_log_detail(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<LegacyLogIdParam>,
) -> Json<LegacyReturn<JobLogDto>> {
    match detail_impl(&state, params.id).await {
        Ok(detail) => Json(LegacyReturn::success_with(detail)),
        Err(err) => Json(LegacyReturn::failure(err.to_string())),
    }
}

async fn legacy_log_detail_post(
    State(state): State<AppState>,
    _user: AuthUser,
    Form(params): Form<LegacyLogIdParam>,
) -> Json<LegacyReturn<JobLogDto>> {
    match detail_impl(&state, params.id).await {
        Ok(detail) => Json(LegacyReturn::success_with(detail)),
        Err(err) => Json(LegacyReturn::failure(err.to_string())),
    }
}

async fn legacy_log_detail_cat(
    State(state): State<AppState>,
    _user: AuthUser,
    Form(params): Form<LegacyLogDetailCatParams>,
) -> Json<LegacyReturn<LogContentDto>> {
    let from = params.from_line_num.unwrap_or(1).max(1);
    match log_content_impl(&state, params.log_id, from).await {
        Ok(content) => Json(LegacyReturn::success_with(content)),
        Err(err) => Json(LegacyReturn::failure(err.to_string())),
    }
}

async fn legacy_log_kill(
    State(state): State<AppState>,
    user: AuthUser,
    Form(params): Form<LegacyKillParams>,
) -> Json<LegacyReturn<String>> {
    match kill_impl(&state, &user, params.id).await {
        Ok(message) => Json(LegacyReturn::success_with(message)),
        Err(err) => Json(LegacyReturn::failure(err.to_string())),
    }
}

async fn legacy_clear_log(
    State(state): State<AppState>,
    user: AuthUser,
    Form(params): Form<LegacyClearRequest>,
) -> Json<LegacyReturn<String>> {
    let payload = match convert_legacy_clear_request(params) {
        Ok(payload) => payload,
        Err(err) => return Json(LegacyReturn::failure(err)),
    };

    match clear_impl(&state, &user, payload).await {
        Ok(_) => Json(LegacyReturn::<String>::success(None)),
        Err(err) => Json(LegacyReturn::failure(err.to_string())),
    }
}

async fn ensure_job_group(state: &AppState, job_group_id: i32) -> AppResult<()> {
    if job_group::Entity::find_by_id(job_group_id)
        .one(state.db())
        .await?
        .is_none()
    {
        return Err(AppError::BadRequest("执行器不存在".into()));
    }
    Ok(())
}

fn parse_time_range(range: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
    let parts: Vec<&str> = range.split(" - ").map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return None;
    }
    let start = NaiveDateTime::parse_from_str(parts[0], "%Y-%m-%d %H:%M:%S").ok()?;
    let end = NaiveDateTime::parse_from_str(parts[1], "%Y-%m-%d %H:%M:%S").ok()?;
    Some((start, end))
}
