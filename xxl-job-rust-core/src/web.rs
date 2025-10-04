use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::db;
use crate::models::{XxlJobGroup, XxlJobInfo, XxlJobUser};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

type DbPool = Arc<Pool<MySql>>;

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

pub fn create_router(pool: Pool<MySql>) -> Router {
    let shared_state = Arc::new(pool);
    Router::new()
        .route("/", get(health_check))
        .route("/job", get(list_jobs).post(add_job))
        .route("/job/:id", get(get_job).put(update_job).delete(delete_job))
        .route("/job_groups", get(list_job_groups))
        .route("/job_group", post(add_job_group))
        .route(
            "/job_group/:id",
            get(get_job_group)
                .put(update_job_group)
                .delete(delete_job_group),
        )
        .route("/job/:job_id/logs", get(list_job_logs))
        .route("/log/:log_id", get(get_job_log))
        .route("/users", get(list_users))
        .route("/user", post(add_user))
        .route("/user/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/login", post(login))
        .route("/dashboard", get(get_dashboard_data))
        .with_state(shared_state)
}

async fn health_check() -> &'static str {
    "Hello, XXL-JOB in Rust!"
}

async fn list_jobs(State(pool): State<DbPool>) -> impl IntoResponse {
    match db::list_jobs(&pool).await {
        Ok(jobs) => (StatusCode::OK, Json(jobs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn add_job(
    State(pool): State<DbPool>,
    Json(job_info): Json<XxlJobInfo>,
) -> impl IntoResponse {
    match db::add_job(&pool, &job_info).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_dashboard_data(State(pool): State<DbPool>) -> impl IntoResponse {
    match db::get_dashboard_data(&pool).await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn list_users(State(pool): State<DbPool>) -> impl IntoResponse {
    match db::list_users(&pool).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn add_user(
    State(pool): State<DbPool>,
    Json(user): Json<XxlJobUser>,
) -> impl IntoResponse {
    // In a real app, you'd hash the password here
    match db::add_user(&pool, &user).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_user(State(pool): State<DbPool>, Path(id): Path<i32>) -> impl IntoResponse {
    match db::get_user(&pool, id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn update_user(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
    Json(mut user): Json<XxlJobUser>,
) -> impl IntoResponse {
    user.id = id;
    // In a real app, you'd hash the password if it's being changed
    match db::update_user(&pool, &user).await {
        Ok(rows_affected) if rows_affected > 0 => {
            (StatusCode::OK, "User updated").into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_user(State(pool): State<DbPool>, Path(id): Path<i32>) -> impl IntoResponse {
    match db::delete_user(&pool, id).await {
        Ok(rows_affected) if rows_affected > 0 => {
            (StatusCode::NO_CONTENT, "User deleted").into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn login(State(pool): State<DbPool>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    match db::get_user_by_username(&pool, &payload.username).await {
        Ok(Some(user)) => {
            // In a real application, you would use a secure password hashing
            // and verification library like argon2 or bcrypt.
            if user.password == payload.password {
                (StatusCode::OK, Json(user)).into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn list_job_logs(
    State(pool): State<DbPool>,
    Path(job_id): Path<i32>,
) -> impl IntoResponse {
    match db::list_job_logs(&pool, job_id).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_job_log(State(pool): State<DbPool>, Path(log_id): Path<i64>) -> impl IntoResponse {
    match db::get_job_log(&pool, log_id).await {
        Ok(Some(log)) => (StatusCode::OK, Json(log)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Log not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn list_job_groups(State(pool): State<DbPool>) -> impl IntoResponse {
    match db::list_job_groups(&pool).await {
        Ok(groups) => (StatusCode::OK, Json(groups)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn add_job_group(
    State(pool): State<DbPool>,
    Json(job_group): Json<XxlJobGroup>,
) -> impl IntoResponse {
    match db::add_job_group(&pool, &job_group).await {
        Ok(id) => (StatusCode::CREATED, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_job_group(State(pool): State<DbPool>, Path(id): Path<i32>) -> impl IntoResponse {
    match db::get_job_group(&pool, id).await {
        Ok(Some(group)) => (StatusCode::OK, Json(group)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Job group not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn update_job_group(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
    Json(mut job_group): Json<XxlJobGroup>,
) -> impl IntoResponse {
    job_group.id = id;
    match db::update_job_group(&pool, &job_group).await {
        Ok(rows_affected) if rows_affected > 0 => {
            (StatusCode::OK, "Job group updated").into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, "Job group not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_job_group(State(pool): State<DbPool>, Path(id): Path<i32>) -> impl IntoResponse {
    match db::delete_job_group(&pool, id).await {
        Ok(rows_affected) if rows_affected > 0 => {
            (StatusCode::NO_CONTENT, "Job group deleted").into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, "Job group not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{self, Body};
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn health_check_works() {
        // We don't have a database, so we can't create a real pool.
        // For this test, we'll create a mock router that doesn't need a database.
        let app = Router::new().route("/", get(health_check));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"Hello, XXL-JOB in Rust!");
    }
}

async fn get_job(State(pool): State<DbPool>, Path(id): Path<i64>) -> impl IntoResponse {
    match db::get_job(&pool, id).await {
        Ok(Some(job)) => (StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn update_job(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(mut job_info): Json<XxlJobInfo>,
) -> impl IntoResponse {
    job_info.id = id;
    match db::update_job(&pool, &job_info).await {
        Ok(rows_affected) if rows_affected > 0 => (StatusCode::OK, "Job updated").into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_job(State(pool): State<DbPool>, Path(id): Path<i64>) -> impl IntoResponse {
    match db::delete_job(&pool, id).await {
        Ok(rows_affected) if rows_affected > 0 => {
            (StatusCode::NO_CONTENT, "Job deleted").into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}