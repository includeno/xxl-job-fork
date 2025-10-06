use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::Local;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthUser;
use crate::entities::{job_info, job_logglue};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:job_id", get(load_glue).post(save_glue))
        .route("/:job_id/versions", get(list_versions))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GlueDto {
    job_id: i32,
    glue_type: String,
    glue_source: Option<String>,
    glue_remark: Option<String>,
    glue_updatetime: Option<chrono::NaiveDateTime>,
}

impl From<job_info::Model> for GlueDto {
    fn from(value: job_info::Model) -> Self {
        Self {
            job_id: value.id,
            glue_type: value.glue_type,
            glue_source: value.glue_source,
            glue_remark: value.glue_remark,
            glue_updatetime: value.glue_updatetime,
        }
    }
}

async fn load_glue(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(job_id): Path<i32>,
) -> AppResult<Json<GlueDto>> {
    let job = job_info::Entity::find_by_id(job_id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;
    Ok(Json(GlueDto::from(job)))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct SaveGlueRequest {
    #[validate(length(min = 1, message = "GLUE 内容不能为空"))]
    glue_source: String,
    #[validate(length(min = 1, max = 128, message = "备注需在 1-128 字符之间"))]
    glue_remark: String,
}

async fn save_glue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(job_id): Path<i32>,
    Json(payload): Json<SaveGlueRequest>,
) -> AppResult<Json<GlueDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let mut job = job_info::Entity::find_by_id(job_id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;

    let now = Local::now().naive_local();
    job.glue_source = Some(payload.glue_source.clone());
    job.glue_remark = Some(payload.glue_remark.clone());
    job.glue_updatetime = Some(now);
    job.update_time = Some(now);

    let active: job_info::ActiveModel = job.clone().into();
    let updated = active.update(state.db()).await?;

    let log = job_logglue::ActiveModel {
        job_id: Set(job_id),
        glue_type: Set(Some(updated.glue_type.clone())),
        glue_source: Set(Some(payload.glue_source.clone())),
        glue_remark: Set(payload.glue_remark.clone()),
        add_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    job_logglue::Entity::insert(log).exec(state.db()).await?;

    Ok(Json(GlueDto::from(updated)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GlueVersionDto {
    id: i32,
    glue_type: Option<String>,
    glue_source: Option<String>,
    glue_remark: String,
    add_time: Option<chrono::NaiveDateTime>,
    update_time: Option<chrono::NaiveDateTime>,
}

impl From<job_logglue::Model> for GlueVersionDto {
    fn from(value: job_logglue::Model) -> Self {
        Self {
            id: value.id,
            glue_type: value.glue_type,
            glue_source: value.glue_source,
            glue_remark: value.glue_remark,
            add_time: value.add_time,
            update_time: value.update_time,
        }
    }
}

async fn list_versions(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(job_id): Path<i32>,
) -> AppResult<Json<Vec<GlueVersionDto>>> {
    let versions = job_logglue::Entity::find()
        .filter(job_logglue::Column::JobId.eq(job_id))
        .order_by_desc(job_logglue::Column::AddTime)
        .all(state.db())
        .await?
        .into_iter()
        .map(GlueVersionDto::from)
        .collect();
    Ok(Json(versions))
}
