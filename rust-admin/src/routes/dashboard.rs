use axum::{extract::State, routing::get, Json, Router};
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use sea_orm::{query::*, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::auth::AuthUser;
use crate::entities::{job_group, job_info, job_log, job_log_report};
use crate::error::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/summary", get(summary))
        .route("/chart", get(chart))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SummaryDto {
    group_count: u64,
    job_count: u64,
    log_total_count: u64,
    log_running_count: i64,
    log_success_count: i64,
    log_fail_count: i64,
}

async fn summary(State(state): State<AppState>, _user: AuthUser) -> AppResult<Json<SummaryDto>> {
    let group_count = job_group::Entity::find().count(state.db()).await? as u64;
    let job_count = job_info::Entity::find().count(state.db()).await? as u64;
    let log_total_count = job_log::Entity::find().count(state.db()).await? as u64;

    let stats = job_log_report::Entity::find()
        .order_by_desc(job_log_report::Column::TriggerDay)
        .one(state.db())
        .await?
        .unwrap_or(job_log_report::Model {
            id: 0,
            trigger_day: None,
            running_count: 0,
            suc_count: 0,
            fail_count: 0,
            update_time: None,
        });

    Ok(Json(SummaryDto {
        group_count,
        job_count,
        log_total_count,
        log_running_count: stats.running_count as i64,
        log_success_count: stats.suc_count as i64,
        log_fail_count: stats.fail_count as i64,
    }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartPoint {
    trigger_day: String,
    running_count: i32,
    suc_count: i32,
    fail_count: i32,
}

async fn chart(State(state): State<AppState>, _user: AuthUser) -> AppResult<Json<Vec<ChartPoint>>> {
    let today = Local::now().date_naive();
    let start_day = today - Duration::days(7);

    let rows = job_log_report::Entity::find()
        .filter(job_log_report::Column::TriggerDay.gte(start_day.and_hms_opt(0, 0, 0).unwrap()))
        .order_by_asc(job_log_report::Column::TriggerDay)
        .all(state.db())
        .await?;

    let mut result = Vec::new();
    for row in rows {
        let day = row.trigger_day.unwrap_or_else(|| {
            NaiveDateTime::new(today, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        });
        result.push(ChartPoint {
            trigger_day: day.format("%Y-%m-%d").to_string(),
            running_count: row.running_count,
            suc_count: row.suc_count,
            fail_count: row.fail_count,
        });
    }

    Ok(Json(result))
}
