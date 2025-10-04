use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;
use dotenvy::dotenv;

use crate::models::XxlJobInfo;

pub async fn init_db_pool() -> Result<Pool<MySql>, sqlx::Error> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
}

pub async fn add_job(pool: &Pool<MySql>, job_info: &XxlJobInfo) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO xxl_job_info (
            job_group, job_desc, add_time, update_time, author, alarm_email, schedule_type, schedule_conf,
            misfire_strategy, executor_route_strategy, executor_handler, executor_param,
            executor_block_strategy, executor_timeout, executor_fail_retry_count,
            glue_type, glue_source, glue_remark, glue_updatetime, child_jobid, trigger_status,
            trigger_last_time, trigger_next_time
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(job_info.job_group)
    .bind(&job_info.job_desc)
    .bind(job_info.add_time)
    .bind(job_info.update_time)
    .bind(&job_info.author)
    .bind(&job_info.alarm_email)
    .bind(&job_info.schedule_type)
    .bind(&job_info.schedule_conf)
    .bind(&job_info.misfire_strategy)
    .bind(&job_info.executor_route_strategy)
    .bind(&job_info.executor_handler)
    .bind(&job_info.executor_param)
    .bind(&job_info.executor_block_strategy)
    .bind(job_info.executor_timeout)
    .bind(job_info.executor_fail_retry_count)
    .bind(&job_info.glue_type)
    .bind(&job_info.glue_source)
    .bind(&job_info.glue_remark)
    .bind(job_info.glue_updatetime)
    .bind(&job_info.child_jobid)
    .bind(job_info.trigger_status)
    .bind(job_info.trigger_last_time)
    .bind(job_info.trigger_next_time)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn get_job(pool: &Pool<MySql>, id: i64) -> Result<Option<XxlJobInfo>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobInfo>(
        r#"
        SELECT
            id, job_group, job_desc, add_time, update_time, author, alarm_email,
            schedule_type, schedule_conf, misfire_strategy, executor_route_strategy,
            executor_handler, executor_param, executor_block_strategy, executor_timeout,
            executor_fail_retry_count, glue_type, glue_source, glue_remark,
            glue_updatetime, child_jobid, trigger_status, trigger_last_time, trigger_next_time
        FROM xxl_job_info
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_job(pool: &Pool<MySql>, job_info: &XxlJobInfo) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE xxl_job_info
        SET
            job_group = ?, job_desc = ?, update_time = ?, author = ?, alarm_email = ?,
            schedule_type = ?, schedule_conf = ?, misfire_strategy = ?,
            executor_route_strategy = ?, executor_handler = ?, executor_param = ?,
            executor_block_strategy = ?, executor_timeout = ?, executor_fail_retry_count = ?,
            glue_type = ?, glue_source = ?, glue_remark = ?, glue_updatetime = ?,
            child_jobid = ?, trigger_status = ?, trigger_last_time = ?, trigger_next_time = ?
        WHERE id = ?
        "#,
    )
    .bind(job_info.job_group)
    .bind(&job_info.job_desc)
    .bind(job_info.update_time)
    .bind(&job_info.author)
    .bind(&job_info.alarm_email)
    .bind(&job_info.schedule_type)
    .bind(&job_info.schedule_conf)
    .bind(&job_info.misfire_strategy)
    .bind(&job_info.executor_route_strategy)
    .bind(&job_info.executor_handler)
    .bind(&job_info.executor_param)
    .bind(&job_info.executor_block_strategy)
    .bind(job_info.executor_timeout)
    .bind(job_info.executor_fail_retry_count)
    .bind(&job_info.glue_type)
    .bind(&job_info.glue_source)
    .bind(&job_info.glue_remark)
    .bind(job_info.glue_updatetime)
    .bind(&job_info.child_jobid)
    .bind(job_info.trigger_status)
    .bind(job_info.trigger_last_time)
    .bind(job_info.trigger_next_time)
    .bind(job_info.id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_job(pool: &Pool<MySql>, id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM xxl_job_info WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}