use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_registry")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub registry_group: String,
    pub registry_key: String,
    pub registry_value: String,
    #[sea_orm(column_type = "DateTime")]
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
