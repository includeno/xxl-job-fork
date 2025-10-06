use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{Duration, TimeZone, Utc};
use cron::Schedule;
use sea_orm::{query::*, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::auth::AuthUser;
use crate::entities::{job_group, job_info, job_log, job_registry};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(page_list).post(create))
        .route("/:id", put(update).delete(remove))
        .route("/:id/start", post(start_job))
        .route("/:id/stop", post(stop_job))
        .route("/:id/trigger", post(trigger_job))
        .route("/next-trigger-time", get(next_trigger_time))
}

#[derive(Debug, Deserialize)]
struct PageParams {
    start: Option<u64>,
    length: Option<u64>,
    job_group: i32,
    trigger_status: Option<i8>,
    job_desc: Option<String>,
    executor_handler: Option<String>,
    author: Option<String>,
}

#[derive(Debug, Serialize)]
struct PageResult<T> {
    records_total: u64,
    records_filtered: u64,
    data: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobInfoDto {
    id: i32,
    job_group: i32,
    job_desc: String,
    author: Option<String>,
    alarm_email: Option<String>,
    schedule_type: String,
    schedule_conf: Option<String>,
    misfire_strategy: String,
    executor_route_strategy: Option<String>,
    executor_handler: Option<String>,
    executor_param: Option<String>,
    executor_block_strategy: Option<String>,
    executor_timeout: i32,
    executor_fail_retry_count: i32,
    glue_type: String,
    glue_source: Option<String>,
    glue_remark: Option<String>,
    glue_updatetime: Option<chrono::NaiveDateTime>,
    child_jobid: Option<String>,
    trigger_status: i8,
    trigger_last_time: i64,
    trigger_next_time: i64,
    add_time: Option<chrono::NaiveDateTime>,
    update_time: Option<chrono::NaiveDateTime>,
}

impl From<job_info::Model> for JobInfoDto {
    fn from(value: job_info::Model) -> Self {
        Self {
            id: value.id,
            job_group: value.job_group,
            job_desc: value.job_desc,
            author: value.author,
            alarm_email: value.alarm_email,
            schedule_type: value.schedule_type,
            schedule_conf: value.schedule_conf,
            misfire_strategy: value.misfire_strategy,
            executor_route_strategy: value.executor_route_strategy,
            executor_handler: value.executor_handler,
            executor_param: value.executor_param,
            executor_block_strategy: value.executor_block_strategy,
            executor_timeout: value.executor_timeout,
            executor_fail_retry_count: value.executor_fail_retry_count,
            glue_type: value.glue_type,
            glue_source: value.glue_source,
            glue_remark: value.glue_remark,
            glue_updatetime: value.glue_updatetime,
            child_jobid: value.child_jobid,
            trigger_status: value.trigger_status,
            trigger_last_time: value.trigger_last_time,
            trigger_next_time: value.trigger_next_time,
            add_time: value.add_time,
            update_time: value.update_time,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct SaveJobInfoRequest {
    job_group: i32,
    #[validate(length(min = 1, message = "任务描述不能为空"))]
    job_desc: String,
    author: Option<String>,
    alarm_email: Option<String>,
    schedule_type: String,
    schedule_conf: Option<String>,
    misfire_strategy: String,
    executor_route_strategy: Option<String>,
    executor_handler: Option<String>,
    executor_param: Option<String>,
    executor_block_strategy: Option<String>,
    executor_timeout: Option<i32>,
    executor_fail_retry_count: Option<i32>,
    glue_type: String,
    glue_source: Option<String>,
    glue_remark: Option<String>,
    child_jobid: Option<String>,
}

async fn page_list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<PageParams>,
) -> AppResult<Json<PageResult<JobInfoDto>>> {
    let mut query =
        job_info::Entity::find().filter(job_info::Column::JobGroup.eq(params.job_group));

    if let Some(trigger_status) = params.trigger_status {
        if trigger_status >= 0 {
            query = query.filter(job_info::Column::TriggerStatus.eq(trigger_status));
        }
    }

    if let Some(job_desc) = params.job_desc.filter(|s| !s.trim().is_empty()) {
        query = query.filter(job_info::Column::JobDesc.contains(job_desc.trim()));
    }
    if let Some(handler) = params.executor_handler.filter(|s| !s.trim().is_empty()) {
        query = query.filter(job_info::Column::ExecutorHandler.contains(handler.trim()));
    }
    if let Some(author) = params.author.filter(|s| !s.trim().is_empty()) {
        query = query.filter(job_info::Column::Author.contains(author.trim()));
    }

    let start = params.start.unwrap_or(0);
    let length = params.length.unwrap_or(10);

    let total = query.clone().count(state.db()).await? as u64;
    let data = query
        .order_by_desc(job_info::Column::UpdateTime)
        .offset(start)
        .limit(length)
        .all(state.db())
        .await?
        .into_iter()
        .map(JobInfoDto::from)
        .collect();

    Ok(Json(PageResult {
        records_total: total,
        records_filtered: total,
        data,
    }))
}

async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<SaveJobInfoRequest>,
) -> AppResult<Json<JobInfoDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    ensure_group_exists(&state, payload.job_group).await?;

    let now = Utc::now().naive_utc();
    let active = job_info::ActiveModel {
        job_group: Set(payload.job_group),
        job_desc: Set(payload.job_desc.clone()),
        add_time: Set(Some(now)),
        update_time: Set(Some(now)),
        author: Set(payload.author.clone()),
        alarm_email: Set(payload.alarm_email.clone()),
        schedule_type: Set(payload.schedule_type.clone()),
        schedule_conf: Set(payload.schedule_conf.clone()),
        misfire_strategy: Set(payload.misfire_strategy.clone()),
        executor_route_strategy: Set(payload.executor_route_strategy.clone()),
        executor_handler: Set(payload.executor_handler.clone()),
        executor_param: Set(payload.executor_param.clone()),
        executor_block_strategy: Set(payload.executor_block_strategy.clone()),
        executor_timeout: Set(payload.executor_timeout.unwrap_or_default()),
        executor_fail_retry_count: Set(payload.executor_fail_retry_count.unwrap_or_default()),
        glue_type: Set(payload.glue_type.clone()),
        glue_source: Set(payload.glue_source.clone()),
        glue_remark: Set(payload.glue_remark.clone()),
        glue_updatetime: Set(Some(now)),
        child_jobid: Set(payload.child_jobid.clone()),
        trigger_status: Set(0),
        trigger_last_time: Set(0),
        trigger_next_time: Set(0),
        ..Default::default()
    };

    let inserted = job_info::Entity::insert(active)
        .exec_with_returning(state.db())
        .await?;

    Ok(Json(JobInfoDto::from(inserted)))
}

async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<SaveJobInfoRequest>,
) -> AppResult<Json<JobInfoDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    ensure_group_exists(&state, payload.job_group).await?;

    let mut model = job_info::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;

    model.job_group = payload.job_group;
    model.job_desc = payload.job_desc.clone();
    model.author = payload.author.clone();
    model.alarm_email = payload.alarm_email.clone();
    model.schedule_type = payload.schedule_type.clone();
    model.schedule_conf = payload.schedule_conf.clone();
    model.misfire_strategy = payload.misfire_strategy.clone();
    model.executor_route_strategy = payload.executor_route_strategy.clone();
    model.executor_handler = payload.executor_handler.clone();
    model.executor_param = payload.executor_param.clone();
    model.executor_block_strategy = payload.executor_block_strategy.clone();
    model.executor_timeout = payload.executor_timeout.unwrap_or_default();
    model.executor_fail_retry_count = payload.executor_fail_retry_count.unwrap_or_default();
    model.glue_type = payload.glue_type.clone();
    model.glue_source = payload.glue_source.clone();
    model.glue_remark = payload.glue_remark.clone();
    model.child_jobid = payload.child_jobid.clone();
    model.update_time = Some(Utc::now().naive_utc());

    let active: job_info::ActiveModel = model.into();
    let updated = active.update(state.db()).await?;

    Ok(Json(JobInfoDto::from(updated)))
}

async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin()?;

    let deleted = job_info::Entity::delete_by_id(id).exec(state.db()).await?;
    if deleted.rows_affected == 0 {
        return Err(AppError::NotFound("任务不存在".into()));
    }

    Ok(Json(json!({ "message": "任务已删除" })))
}

async fn start_job(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<JobInfoDto>> {
    user.require_admin()?;
    let mut model = job_info::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;

    let next = compute_next_trigger(&model)?;
    model.trigger_status = 1;
    model.trigger_last_time = Utc::now().timestamp_millis();
    model.trigger_next_time = next.unwrap_or(0);

    let active: job_info::ActiveModel = model.into();
    let updated = active.update(state.db()).await?;
    Ok(Json(JobInfoDto::from(updated)))
}

async fn stop_job(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<JobInfoDto>> {
    user.require_admin()?;
    let mut model = job_info::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;

    model.trigger_status = 0;
    model.trigger_next_time = 0;

    let active: job_info::ActiveModel = model.into();
    let updated = active.update(state.db()).await?;
    Ok(Json(JobInfoDto::from(updated)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TriggerRequest {
    executor_param: Option<String>,
    address_list: Option<String>,
}

async fn trigger_job(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<TriggerRequest>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin()?;

    let mut job = job_info::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;

    let handler = job
        .executor_handler
        .clone()
        .ok_or_else(|| AppError::BadRequest("任务未配置执行器 Handler".into()))?;

    let group = job_group::Entity::find_by_id(job.job_group)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("执行器分组不存在".into()))?;

    let addresses =
        resolve_executor_addresses(&state, &group, payload.address_list.as_deref()).await?;

    let address = addresses
        .first()
        .cloned()
        .ok_or_else(|| AppError::BadRequest("未找到可用的执行器地址".into()))?;

    let now = Utc::now();
    let executor_param = payload
        .executor_param
        .clone()
        .or(job.executor_param.clone());

    let log_active = job_log::ActiveModel {
        job_group: Set(job.job_group),
        job_id: Set(job.id),
        executor_address: Set(Some(address.clone())),
        executor_handler: Set(Some(handler.clone())),
        executor_param: Set(executor_param.clone()),
        executor_sharding_param: Set(None),
        executor_fail_retry_count: Set(job.executor_fail_retry_count),
        trigger_time: Set(Some(now.naive_utc())),
        trigger_code: Set(0),
        trigger_msg: Set(None),
        handle_time: Set(None),
        handle_code: Set(0),
        handle_msg: Set(None),
        alarm_status: Set(0),
        ..Default::default()
    };

    let inserted = job_log::Entity::insert(log_active).exec(state.db()).await?;
    let log_id = inserted.last_insert_id;

    let trigger_param = build_trigger_param(
        &job,
        log_id,
        now.timestamp_millis(),
        handler,
        executor_param,
    );
    let trigger_result = trigger_executor(
        state.http_client(),
        address.as_str(),
        state.settings().executor.access_token(),
        &trigger_param,
    )
    .await;

    let trigger_code;
    let mut trigger_lines = vec![format!(
        "手动触发任务，触发人: {}，执行器地址: {}",
        user.username, address
    )];

    match trigger_result {
        Ok(result) => {
            trigger_code = result.code;
            if let Some(msg) = result.msg.filter(|m| !m.trim().is_empty()) {
                trigger_lines.push(format!("执行器返回信息: {}", msg));
            }
            if let Some(content) = result.content.filter(|c| !c.trim().is_empty()) {
                trigger_lines.push(format!("执行器返回内容: {}", content));
            }
        }
        Err(err) => {
            trigger_code = 500;
            trigger_lines.push(format!("触发执行器失败: {}", err));
        }
    }

    let trigger_msg = trigger_lines.join("<br>");

    let update_log = job_log::ActiveModel {
        id: Set(log_id),
        trigger_code: Set(trigger_code),
        trigger_msg: Set(Some(trigger_msg)),
        ..Default::default()
    };
    job_log::Entity::update(update_log).exec(state.db()).await?;

    job.trigger_last_time = now.timestamp_millis();
    if let Some(next) = compute_next_trigger(&job)? {
        job.trigger_next_time = next;
    }
    let active: job_info::ActiveModel = job.into();
    active.update(state.db()).await?;

    Ok(Json(json!({ "message": "触发成功" })))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TriggerParamPayload {
    job_id: i32,
    executor_handler: String,
    executor_params: String,
    executor_block_strategy: String,
    executor_timeout: i32,
    log_id: i64,
    log_date_time: i64,
    glue_type: String,
    glue_source: String,
    glue_updatetime: i64,
    broadcast_index: i32,
    broadcast_total: i32,
}

fn build_trigger_param(
    job: &job_info::Model,
    log_id: i64,
    log_time: i64,
    handler: String,
    executor_param: Option<String>,
) -> TriggerParamPayload {
    let executor_params = executor_param.unwrap_or_default();
    let block_strategy = job
        .executor_block_strategy
        .clone()
        .unwrap_or_else(|| "SERIAL_EXECUTION".to_string());
    let glue_source = job.glue_source.clone().unwrap_or_default();
    let glue_updatetime = job
        .glue_updatetime
        .map(|dt| Utc.from_utc_datetime(&dt).timestamp_millis())
        .unwrap_or_default();

    TriggerParamPayload {
        job_id: job.id,
        executor_handler: handler,
        executor_params,
        executor_block_strategy: block_strategy,
        executor_timeout: job.executor_timeout,
        log_id,
        log_date_time: log_time,
        glue_type: job.glue_type.clone(),
        glue_source,
        glue_updatetime,
        broadcast_index: 0,
        broadcast_total: 1,
    }
}

#[derive(Debug, Deserialize)]
struct ExecutorReturn<T> {
    code: i32,
    msg: Option<T>,
    content: Option<T>,
}

async fn trigger_executor(
    client: &reqwest::Client,
    raw_address: &str,
    access_token: Option<&str>,
    payload: &TriggerParamPayload,
) -> anyhow::Result<ExecutorReturn<String>> {
    let mut address = raw_address.trim().to_string();
    if !address.ends_with('/') {
        address.push('/');
    }
    address.push_str("run");

    let mut request = client.post(address).json(payload);
    if let Some(token) = access_token {
        request = request.header("XXL-JOB-ACCESS-TOKEN", token);
    }

    let response = request.send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "执行器返回非成功状态码: {}",
            response.status()
        ));
    }

    Ok(response.json::<ExecutorReturn<String>>().await?)
}

async fn resolve_executor_addresses(
    state: &AppState,
    group: &job_group::Model,
    override_address: Option<&str>,
) -> AppResult<Vec<String>> {
    if let Some(list) = override_address.and_then(parse_address_list) {
        if !list.is_empty() {
            return Ok(list);
        }
    }

    if group.address_type == 1 {
        if let Some(list) = group.address_list.as_deref().and_then(parse_address_list) {
            if !list.is_empty() {
                return Ok(list);
            }
        }
    }

    let registries = job_registry::Entity::find()
        .filter(job_registry::Column::RegistryGroup.eq("EXECUTOR"))
        .filter(job_registry::Column::RegistryKey.eq(group.app_name.as_str()))
        .all(state.db())
        .await?;

    let list: Vec<String> = registries
        .into_iter()
        .map(|item| item.registry_value)
        .filter(|value| !value.trim().is_empty())
        .collect();

    if list.is_empty() {
        return Err(AppError::BadRequest(
            "未检测到可用的执行器实例，请确认执行器是否注册成功".into(),
        ));
    }

    Ok(list)
}

fn parse_address_list(input: &str) -> Option<Vec<String>> {
    let values: Vec<String> = input
        .split([',', '\n'])
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect();

    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

#[derive(Debug, Deserialize)]
struct NextTriggerParams {
    #[serde(rename = "scheduleType")]
    schedule_type: String,
    #[serde(rename = "scheduleConf")]
    schedule_conf: Option<String>,
}

async fn next_trigger_time(
    Query(params): Query<NextTriggerParams>,
) -> AppResult<Json<Vec<String>>> {
    let job = job_info::Model {
        id: 0,
        job_group: 0,
        job_desc: String::new(),
        add_time: None,
        update_time: None,
        author: None,
        alarm_email: None,
        schedule_type: params.schedule_type.clone(),
        schedule_conf: params.schedule_conf.clone(),
        misfire_strategy: String::new(),
        executor_route_strategy: None,
        executor_handler: None,
        executor_param: None,
        executor_block_strategy: None,
        executor_timeout: 0,
        executor_fail_retry_count: 0,
        glue_type: String::new(),
        glue_source: None,
        glue_remark: None,
        glue_updatetime: None,
        child_jobid: None,
        trigger_status: 0,
        trigger_last_time: 0,
        trigger_next_time: 0,
    };

    let mut result = Vec::new();
    let mut last = Utc::now();
    for _ in 0..5 {
        match compute_next_for_params(&job, last) {
            Ok(Some(next)) => {
                let dt = Utc
                    .timestamp_millis_opt(next)
                    .single()
                    .ok_or_else(|| AppError::BadRequest("无法计算下一次调度时间".into()))?;
                result.push(dt.format("%Y-%m-%d %H:%M:%S").to_string());
                last = dt;
            }
            Ok(None) => break,
            Err(err) => return Err(err),
        }
    }

    Ok(Json(result))
}

async fn ensure_group_exists(state: &AppState, group_id: i32) -> AppResult<()> {
    if job_group::Entity::find_by_id(group_id)
        .one(state.db())
        .await?
        .is_none()
    {
        return Err(AppError::BadRequest(format!("执行器 {group_id} 不存在")));
    }
    Ok(())
}

fn compute_next_trigger(job: &job_info::Model) -> AppResult<Option<i64>> {
    compute_next_for_params(job, Utc::now())
}

fn compute_next_for_params(
    job: &job_info::Model,
    after: chrono::DateTime<Utc>,
) -> AppResult<Option<i64>> {
    match job.schedule_type.as_str() {
        "NONE" => Ok(None),
        "CRON" => {
            let expr = job
                .schedule_conf
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("CRON 调度需要提供 scheduleConf".into()))?;
            let schedule = Schedule::from_str(expr)
                .map_err(|err| AppError::BadRequest(format!("CRON 表达式无效: {err}")))?;
            let mut iter = schedule.after(&after);
            if let Some(datetime) = iter.next() {
                Ok(Some(datetime.timestamp_millis()))
            } else {
                Ok(None)
            }
        }
        "FIX_RATE" | "FIX_DELAY" => {
            let interval = job
                .schedule_conf
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("固定频率调度需要 scheduleConf".into()))?;
            let seconds: i64 = interval
                .parse()
                .map_err(|_| AppError::BadRequest("固定频率配置必须是数字".into()))?;
            let target = after + Duration::seconds(seconds);
            Ok(Some(target.timestamp_millis()))
        }
        other => Err(AppError::BadRequest(format!("不支持的调度类型: {other}"))),
    }
}
