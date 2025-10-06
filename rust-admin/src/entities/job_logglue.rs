use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "xxl_job_logglue")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub job_id: i32,
    pub glue_type: Option<String>,
    pub glue_source: Option<String>,
    pub glue_remark: String,
    #[sea_orm(column_type = "DateTime")]
    pub add_time: Option<chrono::NaiveDateTime>,
    #[sea_orm(column_type = "DateTime")]
    pub update_time: Option<chrono::NaiveDateTime>,
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
