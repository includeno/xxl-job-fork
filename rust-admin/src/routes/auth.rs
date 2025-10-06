use axum::{extract::State, routing::post, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use validator::Validate;

use crate::auth::AuthUser;
use crate::entities::job_user;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
}

#[derive(Debug, Deserialize, Validate)]
struct LoginRequest {
    #[validate(length(min = 1, message = "用户名不能为空"))]
    username: String,
    #[validate(length(min = 1, message = "密码不能为空"))]
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user_id: i32,
    username: String,
    role: i8,
    permission: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    payload
        .validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let user = job_user::Entity::find()
        .filter(job_user::Column::Username.eq(payload.username.trim()))
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".into()))?;

    let hashed = hash_password(payload.password.trim());
    if hashed != user.password {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }

    let token = uuid::Uuid::new_v4().to_string();

    let mut active: job_user::ActiveModel = user.into();
    active.token = Set(Some(token.clone()));
    let updated = active.update(state.db()).await?;

    Ok(Json(LoginResponse {
        token,
        user_id: updated.id,
        username: updated.username,
        role: updated.role,
        permission: updated.permission,
    }))
}

async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let record = job_user::Entity::find_by_id(user.id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    let mut active: job_user::ActiveModel = record.into();
    active.token = Set(None);
    active.update(state.db()).await?;

    Ok(Json(serde_json::json!({ "message": "已退出登录" })))
}

fn hash_password(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
