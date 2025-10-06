use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::templates::{
    DashboardTemplate, JobGroupsTemplate, JobInfoTemplate, JobLogsTemplate, LoginTemplate,
};

const APP_NAME: &str = "XXL-JOB 管理台";
const SUMMARY_ENDPOINT: &str = "/api/dashboard/summary";
const CHART_ENDPOINT: &str = "/api/dashboard/chart";
const JOB_GROUPS_ENDPOINT: &str = "/api/job-groups";
const JOB_INFO_ENDPOINT: &str = "/api/job-info";
const JOB_INFO_NEXT_TRIGGER_ENDPOINT: &str = "/api/job-info/next-trigger-time";
const JOB_LOGS_ENDPOINT: &str = "/api/job-logs";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(login_page))
        .route("/dashboard", get(dashboard_page))
        .route("/jobs", get(job_info_page))
        .route("/groups", get(job_groups_page))
        .route("/logs", get(job_logs_page))
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
        active_nav: "dashboard",
        summary_endpoint: SUMMARY_ENDPOINT,
        chart_endpoint: CHART_ENDPOINT,
    })
}

async fn job_info_page() -> AppResult<Html<String>> {
    render_template(JobInfoTemplate {
        app_name: APP_NAME,
        active_nav: "jobs",
        job_groups_endpoint: JOB_GROUPS_ENDPOINT,
        job_info_endpoint: JOB_INFO_ENDPOINT,
        job_info_next_trigger_endpoint: JOB_INFO_NEXT_TRIGGER_ENDPOINT,
    })
}

async fn job_groups_page() -> AppResult<Html<String>> {
    render_template(JobGroupsTemplate {
        app_name: APP_NAME,
        active_nav: "groups",
        job_groups_endpoint: JOB_GROUPS_ENDPOINT,
    })
}

async fn job_logs_page() -> AppResult<Html<String>> {
    render_template(JobLogsTemplate {
        app_name: APP_NAME,
        active_nav: "logs",
        job_groups_endpoint: JOB_GROUPS_ENDPOINT,
        job_logs_endpoint: JOB_LOGS_ENDPOINT,
    })
}

fn render_template<T>(template: T) -> AppResult<Html<String>>
where
    T: Template,
{
    template.render().map(Html).map_err(AppError::internal)
}
