use std::net::SocketAddr;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub security: SecuritySettings,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("RUST_ADMIN").separator("__"));

        if let Ok(path) = std::env::var("RUST_ADMIN_CONFIG") {
            if !path.trim().is_empty() {
                builder =
                    builder.add_source(config::File::with_name(path.as_str()).required(false));
            }
        }

        builder
            .build()
            .context("加载配置失败")?
            .try_deserialize()
            .context("解析配置失败")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl ServerSettings {
    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        let addr = format!("{}:{}", self.host, self.port);
        addr.parse().context("解析服务监听地址失败")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
    pub token_ttl_minutes: i64,
}
