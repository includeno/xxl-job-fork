use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthUser;
use crate::entities::job_user;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", put(update_user).delete(remove_user))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobUserDto {
    id: i32,
    username: String,
    role: i8,
    permission: Option<String>,
}

impl From<job_user::Model> for JobUserDto {
    fn from(value: job_user::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role,
            permission: value.permission,
        }
    }
}

async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<JobUserDto>>> {
    user.require_admin()?;
    let users = job_user::Entity::find()
        .order_by_asc(job_user::Column::Id)
        .all(state.db())
        .await?
        .into_iter()
        .map(JobUserDto::from)
        .collect();
    Ok(Json(users))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct SaveUserRequest {
    #[validate(length(min = 4, max = 50, message = "用户名需为 4-50 位"))]
    username: String,
    #[validate(length(min = 6, message = "密码至少 6 位"))]
    password: Option<String>,
    role: i8,
    permission: Option<String>,
}

async fn create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<SaveUserRequest>,
) -> AppResult<Json<JobUserDto>> {
    user.require_admin()?;
    if payload.password.is_none() {
        return Err(AppError::BadRequest("创建用户必须提供密码".into()));
    }
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if !matches!(payload.role, 0 | 1) {
        return Err(AppError::BadRequest("角色取值非法".into()));
    }

    let SaveUserRequest {
        username,
        password,
        role,
        permission,
    } = payload;

    if job_user::Entity::find()
        .filter(job_user::Column::Username.eq(username.trim()))
        .one(state.db())
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("用户名已存在".into()));
    }

    let password = hash_password(password.unwrap().trim());
    let active = job_user::ActiveModel {
        username: Set(username.trim().to_string()),
        password: Set(password),
        token: Set(None),
        role: Set(role),
        permission: Set(permission.clone()),
        ..Default::default()
    };

    let inserted = job_user::Entity::insert(active)
        .exec_with_returning(state.db())
        .await?;
    Ok(Json(JobUserDto::from(inserted)))
}

async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<SaveUserRequest>,
) -> AppResult<Json<JobUserDto>> {
    user.require_admin()?;
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if !matches!(payload.role, 0 | 1) {
        return Err(AppError::BadRequest("角色取值非法".into()));
    }

    let SaveUserRequest {
        username,
        password,
        role,
        permission,
    } = payload;

    let mut model = job_user::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    if username != model.username {
        if job_user::Entity::find()
            .filter(job_user::Column::Username.eq(username.trim()))
            .one(state.db())
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("用户名已存在".into()));
        }
    }

    model.username = username.trim().to_string();
    model.role = role;
    model.permission = permission.clone();
    if let Some(password) = password {
        model.password = hash_password(password.trim());
        model.token = None;
    }

    let active: job_user::ActiveModel = model.into();
    let updated = active.update(state.db()).await?;
    Ok(Json(JobUserDto::from(updated)))
}

async fn remove_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin()?;
    if id == user.id {
        return Err(AppError::BadRequest("不能删除当前登录用户".into()));
    }

    let result = job_user::Entity::delete_by_id(id).exec(state.db()).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("用户不存在".into()));
    }
    Ok(Json(serde_json::json!({ "message": "用户已删除" })))
}

fn hash_password(password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}
