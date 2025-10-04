use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::db;
use crate::models::XxlJobInfo;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

type DbPool = Arc<Pool<MySql>>;

pub fn create_router(pool: Pool<MySql>) -> Router {
    let shared_state = Arc::new(pool);
    Router::new()
        .route("/", get(health_check))
        .route("/job", post(add_job))
        .route("/job/:id", get(get_job).put(update_job).delete(delete_job))
        .with_state(shared_state)
}

async fn health_check() -> &'static str {
    "Hello, XXL-JOB in Rust!"
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