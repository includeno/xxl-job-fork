use std::env;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Serialize;

#[path = "../request_preview.rs"]
mod request_preview;

use request_preview::format_executor_request_curl;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TriggerParamPayload {
    job_id: i32,
    executor_handler: String,
    executor_params: String,
    executor_block_strategy: String,
    executor_timeout: i32,
    log_id: i64,
    log_date_time: i64,
    glue_type: String,
    glue_source: String,
    glue_updatetime: i64,
    broadcast_index: i32,
    broadcast_total: i32,
}

#[derive(Debug, Clone)]
struct RequestConfig {
    url: String,
    token: Option<String>,
    payload: TriggerParamPayload,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            url: "http://198.18.0.1:9999/run".to_string(),
            token: Some("default_token".to_string()),
            payload: TriggerParamPayload {
                job_id: 1,
                executor_handler: "demoJobHandler".to_string(),
                executor_params: String::new(),
                executor_block_strategy: "SERIAL_EXECUTION".to_string(),
                executor_timeout: 0,
                log_id: 27,
                log_date_time: 1_759_772_157_428,
                glue_type: "BEAN".to_string(),
                glue_source: String::new(),
                glue_updatetime: 1_759_735_819_000,
                broadcast_index: 0,
                broadcast_total: 1,
            },
        }
    }
}

impl RequestConfig {
    fn apply_override(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "jobId" => {
                self.payload.job_id = value
                    .parse()
                    .with_context(|| format!("无法解析 jobId='{}' 为整数", value))?;
            }
            "executorHandler" => self.payload.executor_handler = value.to_string(),
            "executorParams" => self.payload.executor_params = value.to_string(),
            "executorBlockStrategy" => {
                self.payload.executor_block_strategy = value.to_string();
            }
            "executorTimeout" => {
                self.payload.executor_timeout = value
                    .parse()
                    .with_context(|| format!("无法解析 executorTimeout='{}' 为整数", value))?;
            }
            "logId" => {
                self.payload.log_id = value
                    .parse()
                    .with_context(|| format!("无法解析 logId='{}' 为整数", value))?;
            }
            "logDateTime" => {
                self.payload.log_date_time = value
                    .parse()
                    .with_context(|| format!("无法解析 logDateTime='{}' 为整数", value))?;
            }
            "glueType" => self.payload.glue_type = value.to_string(),
            "glueSource" => self.payload.glue_source = value.to_string(),
            "glueUpdatetime" => {
                self.payload.glue_updatetime = value
                    .parse()
                    .with_context(|| format!("无法解析 glueUpdatetime='{}' 为整数", value))?;
            }
            "broadcastIndex" => {
                self.payload.broadcast_index = value
                    .parse()
                    .with_context(|| format!("无法解析 broadcastIndex='{}' 为整数", value))?;
            }
            "broadcastTotal" => {
                self.payload.broadcast_total = value
                    .parse()
                    .with_context(|| format!("无法解析 broadcastTotal='{}' 为整数", value))?;
            }
            other => {
                return Err(anyhow!(
                    "未知参数 '{}'，支持的字段包括 jobId、executorHandler、executorParams 等",
                    other
                ));
            }
        }

        Ok(())
    }
}

fn print_usage() {
    eprintln!(
        "用法: cargo run --bin preview_trigger_request [--url=URL] [--token=TOKEN] [--send] [字段=值...]"
    );
    eprintln!("例如: cargo run --bin preview_trigger_request executorParams=foo logId=42");
    eprintln!(
        "支持覆盖的字段: jobId executorHandler executorParams executorBlockStrategy executorTimeout \
logId logDateTime glueType glueSource glueUpdatetime broadcastIndex broadcastTotal"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let mut config = RequestConfig::default();
    let mut send_request = false;

    for arg in env::args().skip(1) {
        if arg == "--send" {
            send_request = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--url=") {
            config.url = value.to_string();
            continue;
        }

        if let Some(value) = arg.strip_prefix("--token=") {
            config.token = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            continue;
        }

        let Some((key, value)) = arg.split_once('=') else {
            return Err(anyhow!(
                "参数 '{}' 格式不正确，应为 key=value 或 --url= --token=",
                arg
            ));
        };

        config.apply_override(key, value)?;
    }

    let body_compact = serde_json::to_string(&config.payload)?;
    let pretty_body =
        request_preview::to_pretty_json(&config.payload).unwrap_or_else(|| body_compact.clone());
    let curl = format_executor_request_curl(&config.url, config.token.as_deref(), &body_compact);

    println!("=== HTTP 请求 ===");
    println!("POST {} HTTP/1.1", config.url);
    println!("Content-Type: application/json");
    if let Some(token) = config.token.as_deref() {
        if !token.trim().is_empty() {
            println!("XXL-JOB-ACCESS-TOKEN: {}", token.trim());
        }
    }
    println!();
    println!("{}", pretty_body);
    println!();
    println!("=== curl 命令 ===");
    println!("{}", curl);

    if send_request {
        println!();
        println!("=== 实际请求响应 ===");

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("构建 HTTP 客户端失败")?;

        let mut request = client.post(&config.url).header("Content-Type", "application/json");
        if let Some(token) = config
            .token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request = request.header("XXL-JOB-ACCESS-TOKEN", token);
        }

        let response = request
            .body(body_compact.clone())
            .send()
            .await
            .with_context(|| format!("向 {} 发送请求失败", config.url))?;

        let status = response.status();
        let status_reason = status.canonical_reason().map(str::to_string);
        let headers = response.headers().clone();
        let body_text = response.text().await.context("读取响应体失败")?;

        println!("状态: {} {:?}", status, status_reason.as_deref());
        println!("--- 响应头 ---");
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                println!("{}: {}", name, value_str);
            } else {
                println!("{}: <非 UTF-8 值>", name);
            }
        }

        println!("--- 响应体 ---");
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                println!("{}", pretty_json);
            } else {
                println!("{}", body_text);
            }
        } else {
            println!("{}", body_text);
        }
    }

    Ok(())
}
