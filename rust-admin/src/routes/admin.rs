use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::templates::{DashboardTemplate, LoginTemplate};

const APP_NAME: &str = "XXL-JOB 管理台";
const SUMMARY_ENDPOINT: &str = "/api/dashboard/summary";
const CHART_ENDPOINT: &str = "/api/dashboard/chart";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(login_page))
        .route("/dashboard", get(dashboard_page))
}

async fn login_page() -> AppResult<Html<String>> {
    render_template(LoginTemplate {
        app_name: APP_NAME,
        tagline: "轻量级分布式任务调度中心",
    })
}

async fn dashboard_page() -> AppResult<Html<String>> {
    render_template(DashboardTemplate {
        app_name: APP_NAME,
        summary_endpoint: SUMMARY_ENDPOINT,
        chart_endpoint: CHART_ENDPOINT,
    })
}

fn render_template<T>(template: T) -> AppResult<Html<String>>
where
    T: Template,
{
    template.render().map(Html).map_err(AppError::internal)
}
