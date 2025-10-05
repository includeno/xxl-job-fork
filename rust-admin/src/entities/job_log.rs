use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_group: i32,
    pub job_id: i32,
    pub executor_address: Option<String>,
    pub executor_handler: Option<String>,
    pub executor_param: Option<String>,
    pub executor_sharding_param: Option<String>,
    pub executor_fail_retry_count: i32,
    #[sea_orm(column_type = "DateTime")]
    pub trigger_time: Option<chrono::NaiveDateTime>,
    pub trigger_code: i32,
    pub trigger_msg: Option<String>,
    #[sea_orm(column_type = "DateTime")]
    pub handle_time: Option<chrono::NaiveDateTime>,
    pub handle_code: i32,
    pub handle_msg: Option<String>,
    #[sea_orm(column_type = "TinyInteger")]
    pub alarm_status: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::job_info::Entity",
        from = "Column::JobId",
        to = "super::job_info::Column::Id"
    )]
    JobInfo,
}

impl Related<super::job_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobInfo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
