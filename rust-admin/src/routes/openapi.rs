use axum::{extract::State, routing::post, Json, Router};
use chrono::Local;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::entities::{job_log, job_registry};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/registry", post(registry))
        .route("/registryRemove", post(registry_remove))
        .route("/callback", post(callback))
}

#[derive(Debug, Serialize)]
struct ReturnT<T> {
    code: i32,
    msg: Option<String>,
    content: Option<T>,
}

impl<T> ReturnT<T> {
    fn success(content: Option<T>) -> Self {
        Self {
            code: 200,
            msg: None,
            content,
        }
    }

    fn fail(message: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: Some(message.into()),
            content: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryRequest {
    registry_group: String,
    registry_key: String,
    registry_value: String,
}

async fn registry(
    State(state): State<AppState>,
    Json(payload): Json<RegistryRequest>,
) -> Json<ReturnT<String>> {
    match save_registry(&state, payload).await {
        Ok(_) => Json(ReturnT::success(None)),
        Err(err) => Json(ReturnT::fail(err)),
    }
}

async fn save_registry(state: &AppState, payload: RegistryRequest) -> Result<(), String> {
    let group = payload.registry_group.trim();
    let key = payload.registry_key.trim();
    let value = payload.registry_value.trim();

    if group.is_empty() || key.is_empty() || value.is_empty() {
        return Err("registryGroup/registryKey/registryValue 不能为空".into());
    }

    let now = Local::now().naive_local();
    let existing = job_registry::Entity::find()
        .filter(job_registry::Column::RegistryGroup.eq(group))
        .filter(job_registry::Column::RegistryKey.eq(key))
        .filter(job_registry::Column::RegistryValue.eq(value))
        .one(state.db())
        .await
        .map_err(|err| {
            error!("查询执行器注册信息失败: {err}");
            "保存执行器注册信息失败".to_string()
        })?;

    if let Some(mut model) = existing {
        model.update_time = Some(now);
        let active: job_registry::ActiveModel = model.into();
        active.update(state.db()).await.map_err(|err| {
            error!("更新执行器注册信息失败: {err}");
            "保存执行器注册信息失败".to_string()
        })?;
    } else {
        let active = job_registry::ActiveModel {
            registry_group: Set(group.to_string()),
            registry_key: Set(key.to_string()),
            registry_value: Set(value.to_string()),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        active.insert(state.db()).await.map_err(|err| {
            error!("新增执行器注册信息失败: {err}");
            "保存执行器注册信息失败".to_string()
        })?;
    }

    Ok(())
}

async fn registry_remove(
    State(state): State<AppState>,
    Json(payload): Json<RegistryRequest>,
) -> Json<ReturnT<String>> {
    match remove_registry(&state, payload).await {
        Ok(_) => Json(ReturnT::success(None)),
        Err(err) => Json(ReturnT::fail(err)),
    }
}

async fn remove_registry(state: &AppState, payload: RegistryRequest) -> Result<(), String> {
    let group = payload.registry_group.trim();
    let key = payload.registry_key.trim();
    let value = payload.registry_value.trim();

    if group.is_empty() || key.is_empty() || value.is_empty() {
        return Err("registryGroup/registryKey/registryValue 不能为空".into());
    }

    job_registry::Entity::delete_many()
        .filter(job_registry::Column::RegistryGroup.eq(group))
        .filter(job_registry::Column::RegistryKey.eq(key))
        .filter(job_registry::Column::RegistryValue.eq(value))
        .exec(state.db())
        .await
        .map_err(|err| {
            error!("删除执行器注册信息失败: {err}");
            "删除执行器注册信息失败".to_string()
        })?;

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HandleCallbackParam {
    log_id: i64,
    log_date_tim: i64,
    handle_code: i32,
    handle_msg: Option<String>,
}

async fn callback(
    State(state): State<AppState>,
    Json(params): Json<Vec<HandleCallbackParam>>,
) -> Json<ReturnT<String>> {
    for param in params {
        if let Err(err) = process_callback(&state, param).await {
            warn!("处理执行回调失败: {err}");
        }
    }

    Json(ReturnT::success(None))
}

async fn process_callback(state: &AppState, param: HandleCallbackParam) -> Result<(), String> {
    let mut model = job_log::Entity::find_by_id(param.log_id)
        .one(state.db())
        .await
        .map_err(|err| {
            error!("查询调度日志失败: {err}");
            "查询调度日志失败".to_string()
        })?
        .ok_or_else(|| "log item not found.".to_string())?;

    if model.handle_code > 0 {
        return Err("log repeate callback.".into());
    }

    let mut combined_msg = model.handle_msg.unwrap_or_default();
    if let Some(msg) = param.handle_msg {
        if !combined_msg.is_empty() {
            combined_msg.push('\n');
        }
        combined_msg.push_str(msg.trim());
    }

    model.handle_time = Some(Local::now().naive_local());
    model.handle_code = param.handle_code;
    model.handle_msg = if combined_msg.is_empty() {
        None
    } else {
        Some(combined_msg)
    };

    let active: job_log::ActiveModel = model.into();
    active.update(state.db()).await.map_err(|err| {
        error!("更新调度日志失败: {err}");
        "更新调度日志失败".to_string()
    })?;

    Ok(())
}
