use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use sea_orm::{query::*, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthUser;
use crate::entities::{job_group, job_info, job_registry};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(page_list).post(create))
        .route("/:id", get(load).put(update).delete(remove))
}

#[derive(Debug, Deserialize)]
struct PageParams {
    start: Option<u64>,
    length: Option<u64>,
    appname: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct PageResult<T> {
    records_total: u64,
    records_filtered: u64,
    data: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobGroupDto {
    id: i32,
    appname: String,
    title: String,
    address_type: i8,
    address_list: Option<String>,
    update_time: Option<chrono::NaiveDateTime>,
}

impl From<job_group::Model> for JobGroupDto {
    fn from(value: job_group::Model) -> Self {
        Self {
            id: value.id,
            appname: value.app_name,
            title: value.title,
            address_type: value.address_type,
            address_list: value.address_list,
            update_time: value.update_time,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct SaveJobGroupRequest {
    #[validate(length(min = 4, max = 64, message = "AppName 长度需在 4-64 之间"))]
    appname: String,
    #[validate(length(min = 1, message = "执行器标题不能为空"))]
    title: String,
    address_type: i8,
    address_list: Option<String>,
}

async fn page_list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<PageParams>,
) -> AppResult<Json<PageResult<JobGroupDto>>> {
    let mut query = job_group::Entity::find();

    if let Some(appname) = params.appname.filter(|s| !s.trim().is_empty()) {
        query = query.filter(job_group::Column::AppName.contains(appname.trim()));
    }
    if let Some(title) = params.title.filter(|s| !s.trim().is_empty()) {
        query = query.filter(job_group::Column::Title.contains(title.trim()));
    }

    let start = params.start.unwrap_or(0);
    let length = params.length.unwrap_or(10);

    let total = query.clone().count(state.db()).await? as u64;
    let data = query
        .order_by_asc(job_group::Column::Id)
        .offset(start)
        .limit(length)
        .all(state.db())
        .await?
        .into_iter()
        .map(JobGroupDto::from)
        .collect();

    Ok(Json(PageResult {
        records_total: total,
        records_filtered: total,
        data,
    }))
}

async fn load(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<JobGroupDto>> {
    let model = job_group::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("执行器不存在".into()))?;
    Ok(Json(JobGroupDto::from(model)))
}

async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<SaveJobGroupRequest>,
) -> AppResult<Json<JobGroupDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if payload.appname.contains('<') || payload.appname.contains('>') {
        return Err(AppError::BadRequest("AppName 包含非法字符".into()));
    }
    if payload.title.contains('<') || payload.title.contains('>') {
        return Err(AppError::BadRequest("执行器标题包含非法字符".into()));
    }

    let address_list = validate_address_list(&state, &payload).await?;

    let now = Utc::now().naive_utc();
    let active = job_group::ActiveModel {
        app_name: Set(payload.appname.trim().to_string()),
        title: Set(payload.title.trim().to_string()),
        address_type: Set(payload.address_type),
        address_list: Set(address_list),
        update_time: Set(Some(now)),
        ..Default::default()
    };

    let inserted = job_group::Entity::insert(active)
        .exec_with_returning(state.db())
        .await?;

    Ok(Json(JobGroupDto::from(inserted)))
}

async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<SaveJobGroupRequest>,
) -> AppResult<Json<JobGroupDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let mut model = job_group::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("执行器不存在".into()))?;

    if payload.appname.contains('<') || payload.appname.contains('>') {
        return Err(AppError::BadRequest("AppName 包含非法字符".into()));
    }
    if payload.title.contains('<') || payload.title.contains('>') {
        return Err(AppError::BadRequest("执行器标题包含非法字符".into()));
    }

    let address_list = validate_address_list(&state, &payload).await?;

    model.app_name = payload.appname.trim().to_string();
    model.title = payload.title.trim().to_string();
    model.address_type = payload.address_type;
    model.address_list = address_list;
    model.update_time = Some(Utc::now().naive_utc());

    let active: job_group::ActiveModel = model.into();
    let updated = active.update(state.db()).await?;

    Ok(Json(JobGroupDto::from(updated)))
}

async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin()?;

    let job_count = job_info::Entity::find()
        .filter(job_info::Column::JobGroup.eq(id))
        .count(state.db())
        .await?;
    if job_count > 0 {
        return Err(AppError::BadRequest(
            "该执行器下仍存在任务，无法删除".into(),
        ));
    }

    let group_total = job_group::Entity::find().count(state.db()).await?;
    if group_total <= 1 {
        return Err(AppError::BadRequest("至少保留一个执行器分组".into()));
    }

    let result = job_group::Entity::delete_by_id(id).exec(state.db()).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("执行器不存在".into()));
    }

    Ok(Json(serde_json::json!({ "message": "已删除执行器" })))
}

async fn validate_address_list(
    state: &AppState,
    payload: &SaveJobGroupRequest,
) -> AppResult<Option<String>> {
    if payload.address_type == 0 {
        // 自动注册模式，根据注册表回填
        let addresses = job_registry::Entity::find()
            .filter(job_registry::Column::RegistryGroup.eq("EXECUTOR"))
            .filter(job_registry::Column::RegistryKey.eq(payload.appname.trim()))
            .all(state.db())
            .await?;

        if addresses.is_empty() {
            return Ok(None);
        }
        let mut addrs: Vec<String> = addresses
            .into_iter()
            .map(|item| item.registry_value)
            .collect();
        addrs.sort();
        addrs.dedup();
        Ok(Some(addrs.join(",")))
    } else {
        let list = payload
            .address_list
            .as_ref()
            .map(|s| s.split(',').map(|item| item.trim()).collect::<Vec<_>>())
            .unwrap_or_default();

        if list.is_empty() {
            return Err(AppError::BadRequest(
                "手动录入模式下地址列表不能为空".into(),
            ));
        }
        if list.iter().any(|item| item.is_empty()) {
            return Err(AppError::BadRequest("执行器地址列表包含空值".into()));
        }
        let joined = list.join(",");
        if joined.contains('<') || joined.contains('>') {
            return Err(AppError::BadRequest("执行器地址包含非法字符".into()));
        }
        Ok(Some(joined))
    }
}
