use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("请求参数错误: {0}")]
    BadRequest(String),
    #[error("未认证: {0}")]
    Unauthorized(String),
    #[error("无权限: {0}")]
    Forbidden(String),
    #[error("资源不存在: {0}")]
    NotFound(String),
    #[error("资源冲突: {0}")]
    Conflict(String),
    #[error(transparent)]
    DbError(#[from] DbErr),
    #[error("服务器内部错误")]
    Internal(anyhow::Error),
}

impl AppError {
    pub fn internal<E: Into<anyhow::Error>>(err: E) -> Self {
        Self::Internal(err.into())
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::DbError(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if matches!(self, AppError::DbError(_) | AppError::Internal(_)) {
            error!(error = ?self, "服务器处理请求失败");
        }

        let message = self.to_string();
        let body = Json(ErrorResponse {
            code: status.as_u16(),
            message,
        });

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}
