use axum::{response::Redirect, routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub mod admin;
pub mod auth;
pub mod dashboard;
pub mod glue;
pub mod job_groups;
pub mod job_info;
pub mod job_logs;
pub mod job_user;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root_redirect))
        .nest("/api/auth", auth::router())
        .nest("/api/dashboard", dashboard::router())
        .nest("/api/job-groups", job_groups::router())
        .nest("/api/job-info", job_info::router())
        .nest("/api/job-logs", job_logs::router())
        .nest("/api/job-users", job_user::router())
        .nest("/api/job-code", glue::router())
        .nest("/admin", admin::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

async fn root_redirect() -> Redirect {
    Redirect::permanent("/admin")
}
