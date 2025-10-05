use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_group")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_name = "app_name")]
    pub app_name: String,
    pub title: String,
    #[sea_orm(column_type = "TinyInteger")]
    pub address_type: i8,
    pub address_list: Option<String>,
    #[sea_orm(column_type = "DateTime")]
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::job_info::Entity")]
    JobInfo,
}

impl ActiveModelBehavior for ActiveModel {}
