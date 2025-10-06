use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_log_report")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "DateTime")]
    pub trigger_day: Option<chrono::NaiveDateTime>,
    pub running_count: i32,
    pub suc_count: i32,
    pub fail_count: i32,
    #[sea_orm(column_type = "DateTime")]
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
