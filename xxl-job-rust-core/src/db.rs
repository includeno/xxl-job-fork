use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;
use dotenvy::dotenv;

use crate::models::{DashboardData, XxlJobGroup, XxlJobInfo, XxlJobLog, XxlJobUser};

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

pub async fn list_jobs(pool: &Pool<MySql>) -> Result<Vec<XxlJobInfo>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobInfo>("SELECT * FROM xxl_job_info ORDER BY id DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_dashboard_data(pool: &Pool<MySql>) -> Result<DashboardData, sqlx::Error> {
    let total_jobs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM xxl_job_info")
        .fetch_one(pool)
        .await?;

    let total_job_groups: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM xxl_job_group")
        .fetch_one(pool)
        .await?;

    let recent_success_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM xxl_job_log WHERE handle_code = 200 AND trigger_time >= NOW() - INTERVAL 1 DAY",
    )
    .fetch_one(pool)
    .await?;

    let recent_fail_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM xxl_job_log WHERE handle_code != 200 AND trigger_time >= NOW() - INTERVAL 1 DAY",
    )
    .fetch_one(pool)
    .await?;

    Ok(DashboardData {
        total_jobs: total_jobs.0,
        total_job_groups: total_job_groups.0,
        recent_success_count: recent_success_count.0,
        recent_fail_count: recent_fail_count.0,
    })
}

pub async fn add_user(pool: &Pool<MySql>, user: &XxlJobUser) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO xxl_job_user (username, password, role, permission)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&user.username)
    .bind(&user.password)
    .bind(user.role)
    .bind(&user.permission)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn get_user_by_username(
    pool: &Pool<MySql>,
    username: &str,
) -> Result<Option<XxlJobUser>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobUser>("SELECT * FROM xxl_job_user WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_user(pool: &Pool<MySql>, id: i32) -> Result<Option<XxlJobUser>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobUser>("SELECT * FROM xxl_job_user WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_users(pool: &Pool<MySql>) -> Result<Vec<XxlJobUser>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobUser>("SELECT * FROM xxl_job_user ORDER BY username")
        .fetch_all(pool)
        .await
}

pub async fn update_user(pool: &Pool<MySql>, user: &XxlJobUser) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE xxl_job_user
        SET username = ?, password = ?, role = ?, permission = ?
        WHERE id = ?
        "#,
    )
    .bind(&user.username)
    .bind(&user.password)
    .bind(user.role)
    .bind(&user.permission)
    .bind(user.id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_user(pool: &Pool<MySql>, id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM xxl_job_user WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_job_logs(pool: &Pool<MySql>, job_id: i32) -> Result<Vec<XxlJobLog>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobLog>(
        "SELECT * FROM xxl_job_log WHERE job_id = ? ORDER BY trigger_time DESC",
    )
    .bind(job_id)
    .fetch_all(pool)
    .await
}

pub async fn get_job_log(pool: &Pool<MySql>, log_id: i64) -> Result<Option<XxlJobLog>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobLog>("SELECT * FROM xxl_job_log WHERE id = ?")
        .bind(log_id)
        .fetch_optional(pool)
        .await
}

pub async fn add_job_group(pool: &Pool<MySql>, job_group: &XxlJobGroup) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO xxl_job_group (app_name, title, address_type, address_list, update_time)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&job_group.app_name)
    .bind(&job_group.title)
    .bind(job_group.address_type)
    .bind(&job_group.address_list)
    .bind(job_group.update_time)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn get_job_group(pool: &Pool<MySql>, id: i32) -> Result<Option<XxlJobGroup>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobGroup>(
        "SELECT id, app_name, title, address_type, address_list, update_time FROM xxl_job_group WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_job_groups(pool: &Pool<MySql>) -> Result<Vec<XxlJobGroup>, sqlx::Error> {
    sqlx::query_as::<_, XxlJobGroup>(
        "SELECT id, app_name, title, address_type, address_list, update_time FROM xxl_job_group ORDER BY app_name",
    )
    .fetch_all(pool)
    .await
}

pub async fn update_job_group(pool: &Pool<MySql>, job_group: &XxlJobGroup) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE xxl_job_group
        SET app_name = ?, title = ?, address_type = ?, address_list = ?, update_time = ?
        WHERE id = ?
        "#,
    )
    .bind(&job_group.app_name)
    .bind(&job_group.title)
    .bind(job_group.address_type)
    .bind(&job_group.address_list)
    .bind(job_group.update_time)
    .bind(job_group.id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_job_group(pool: &Pool<MySql>, id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM xxl_job_group WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}