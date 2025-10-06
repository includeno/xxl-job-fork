use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub job_group: i32,
    pub job_desc: String,
    #[sea_orm(column_type = "DateTime")]
    pub add_time: Option<chrono::NaiveDateTime>,
    #[sea_orm(column_type = "DateTime")]
    pub update_time: Option<chrono::NaiveDateTime>,
    pub author: Option<String>,
    pub alarm_email: Option<String>,
    pub schedule_type: String,
    pub schedule_conf: Option<String>,
    pub misfire_strategy: String,
    pub executor_route_strategy: Option<String>,
    pub executor_handler: Option<String>,
    pub executor_param: Option<String>,
    pub executor_block_strategy: Option<String>,
    pub executor_timeout: i32,
    pub executor_fail_retry_count: i32,
    pub glue_type: String,
    pub glue_source: Option<String>,
    pub glue_remark: Option<String>,
    #[sea_orm(column_type = "DateTime")]
    pub glue_updatetime: Option<chrono::NaiveDateTime>,
    pub child_jobid: Option<String>,
    #[sea_orm(column_type = "TinyInteger")]
    pub trigger_status: i8,
    pub trigger_last_time: i64,
    pub trigger_next_time: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::job_group::Entity",
        from = "Column::JobGroup",
        to = "super::job_group::Column::Id"
    )]
    JobGroup,
    #[sea_orm(has_many = "super::job_log::Entity")]
    JobLog,
    #[sea_orm(has_many = "super::job_logglue::Entity")]
    JobLogGlue,
}

impl Related<super::job_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobGroup.def()
    }
}

impl Related<super::job_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobLog.def()
    }
}

impl Related<super::job_logglue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobLogGlue.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
