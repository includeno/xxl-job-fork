use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::entities::job_user;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    User = 0,
    Admin = 1,
}

impl UserRole {
    pub fn from_i8(value: i8) -> AppResult<Self> {
        match value {
            0 => Ok(UserRole::User),
            1 => Ok(UserRole::Admin),
            _ => Err(AppError::BadRequest(format!("未知角色类型: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub role: UserRole,
    pub permission: Option<String>,
}

impl AuthUser {
    pub fn require_admin(&self) -> AppResult<()> {
        if !matches!(self.role, UserRole::Admin) {
            return Err(AppError::Forbidden("需要管理员权限".into()));
        }
        Ok(())
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await;
        let TypedHeader(Authorization(bearer)) = match header {
            Ok(value) => value,
            Err(rejection) => {
                if rejection.is_missing() {
                    return Err(AppError::Unauthorized(
                        "缺少 Authorization Bearer 令牌".into(),
                    ));
                }
                return Err(AppError::Unauthorized("无法解析 Authorization 头".into()));
            }
        };

        let state = AppState::from_ref(state);
        let token = bearer.token();

        let user = job_user::Entity::find()
            .filter(job_user::Column::Token.eq(token))
            .one(state.db())
            .await?
            .ok_or_else(|| AppError::Unauthorized("无效的登录令牌".into()))?;

        let role = UserRole::from_i8(user.role)?;
        Ok(AuthUser {
            id: user.id,
            username: user.username,
            role,
            permission: user.permission,
        })
    }
}
