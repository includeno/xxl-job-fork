use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct XxlJobInfo {
    pub id: i64,
    pub job_group: i32,
    pub job_desc: String,
    pub add_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub author: String,
    pub alarm_email: String,
    pub schedule_type: String,
    pub schedule_conf: String,
    pub misfire_strategy: String,
    pub executor_route_strategy: String,
    pub executor_handler: String,
    pub executor_param: String,
    pub executor_block_strategy: String,
    pub executor_timeout: i32,
    pub executor_fail_retry_count: i32,
    pub glue_type: String,
    pub glue_source: String,
    pub glue_remark: String,
    pub glue_updatetime: Option<DateTime<Utc>>,
    pub child_jobid: String,
    pub trigger_status: i8,
    pub trigger_last_time: i64,
    pub trigger_next_time: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct XxlJobGroup {
    pub id: i32,
    pub app_name: String,
    pub title: String,
    pub address_type: i8,
    pub address_list: Option<String>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct XxlJobLog {
    pub id: i64,
    pub job_group: i32,
    pub job_id: i32,
    pub executor_address: Option<String>,
    pub executor_handler: Option<String>,
    pub executor_param: Option<String>,
    pub executor_sharding_param: Option<String>,
    pub executor_fail_retry_count: Option<i32>,
    pub trigger_time: Option<DateTime<Utc>>,
    pub trigger_code: i32,
    pub trigger_msg: Option<String>,
    pub handle_time: Option<DateTime<Utc>>,
    pub handle_code: i32,
    pub handle_msg: Option<String>,
    pub alarm_status: i8,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct XxlJobUser {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: i8,
    pub permission: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_jobs: i64,
    pub total_job_groups: i64,
    pub recent_success_count: i64,
    pub recent_fail_count: i64,
}