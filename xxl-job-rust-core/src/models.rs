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