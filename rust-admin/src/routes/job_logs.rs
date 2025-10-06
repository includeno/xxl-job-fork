use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, NaiveDateTime, Utc};
use sea_orm::{query::*, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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

async fn page_list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<PageParams>,
) -> AppResult<Json<PageResult<JobLogDto>>> {
    ensure_job_group(&state, params.job_group).await?;

    let mut query = job_log::Entity::find().filter(job_log::Column::JobGroup.eq(params.job_group));

    if let Some(job_id) = params.job_id {
        if job_id > 0 {
            query = query.filter(job_log::Column::JobId.eq(job_id));
        }
    }
    if let Some(status) = params.log_status {
        if status == 1 {
            query = query.filter(job_log::Column::HandleCode.eq(200));
        } else if status == 2 {
            query = query.filter(job_log::Column::HandleCode.ne(200));
        }
    }

    if let Some(range) = params.filter_time.as_ref() {
        if let Some((start, end)) = parse_time_range(range) {
            query = query
                .filter(job_log::Column::TriggerTime.gte(start))
                .filter(job_log::Column::TriggerTime.lte(end));
        }
    }

    let start = params.start.unwrap_or(0);
    let length = params.length.unwrap_or(10);

    let total = query.clone().count(state.db()).await? as u64;
    let data = query
        .order_by_desc(job_log::Column::TriggerTime)
        .offset(start)
        .limit(length)
        .all(state.db())
        .await?
        .into_iter()
        .map(JobLogDto::from)
        .collect();

    Ok(Json(PageResult {
        records_total: total,
        records_filtered: total,
        data,
    }))
}

async fn detail(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<JobLogDto>> {
    let log = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;
    Ok(Json(JobLogDto::from(log)))
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
    let log = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;

    let content = format!(
        "调度日志:\n{}\n执行日志:\n{}",
        log.trigger_msg.unwrap_or_default(),
        log.handle_msg.unwrap_or_default()
    );

    let from = params.from_line_num.unwrap_or(1);
    let total_lines = content.lines().count() as i64;

    Ok(Json(LogContentDto {
        from_line_num: from,
        to_line_num: total_lines,
        end: true,
        log_content: content,
    }))
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
    user.require_admin()?;
    let mut model = job_log::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("日志不存在".into()))?;

    if model.handle_code == 200 {
        return Err(AppError::BadRequest("任务已完成，无需终止".into()));
    }

    model.handle_code = 500;
    model.handle_time = Some(Utc::now().naive_utc());
    let msg = format!("操作人 {} 强制终止任务", user.username);
    model.handle_msg = Some(match model.handle_msg {
        Some(existing) => format!("{}\n{}", existing, msg),
        None => msg,
    });

    let active: job_log::ActiveModel = model.into();
    active.update(state.db()).await?;

    Ok(Json(json!({ "message": "已标记为终止" })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClearRequest {
    job_group: i32,
    job_id: Option<i32>,
    clear_before_days: Option<i64>,
}

async fn clear(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<ClearRequest>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin()?;
    ensure_job_group(&state, payload.job_group).await?;

    let mut delete =
        job_log::Entity::delete_many().filter(job_log::Column::JobGroup.eq(payload.job_group));
    if let Some(job_id) = payload.job_id {
        delete = delete.filter(job_log::Column::JobId.eq(job_id));
    }
    if let Some(days) = payload.clear_before_days {
        if days > 0 {
            let threshold = Utc::now() - Duration::days(days);
            delete = delete.filter(job_log::Column::TriggerTime.lte(threshold.naive_utc()));
        }
    }

    let result = delete.exec(state.db()).await?;
    Ok(Json(json!({ "deleted": result.rows_affected })))
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
