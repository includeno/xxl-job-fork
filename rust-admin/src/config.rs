use std::{net::SocketAddr, time::Duration};

use anyhow::{anyhow, Context};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    #[serde(default)]
    pub database: DatabaseSettings,
    #[serde(default)]
    pub spring: SpringSettings,
    #[serde(default)]
    pub executor: ExecutorSettings,
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

    pub fn database_url(&self) -> anyhow::Result<String> {
        if let Some(url) = self
            .database
            .url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
        {
            if let Some(driver) = self
                .database
                .driver_class_name
                .as_deref()
                .filter(|driver| !driver.trim().is_empty())
            {
                if !driver.to_ascii_lowercase().contains("mysql") {
                    return Err(anyhow!(
                        "当前仅支持 MySQL 数据源，检测到 driver-class-name = `{driver}`"
                    ));
                }
            }

            return build_mysql_url(
                url,
                self.database.username.as_deref(),
                self.database.password.as_deref(),
            );
        }

        let mut datasource = self.spring.datasource.clone();
        datasource.apply_env_overrides();

        if let Some(url) = datasource
            .url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
        {
            if let Some(driver) = datasource
                .driver_class_name
                .as_deref()
                .filter(|driver| !driver.trim().is_empty())
            {
                if !driver.to_ascii_lowercase().contains("mysql") {
                    return Err(anyhow!(
                        "当前仅支持 MySQL 数据源，检测到 driver-class-name = `{driver}`"
                    ));
                }
            }

            return build_mysql_url(
                url,
                datasource.username.as_deref(),
                datasource.password.as_deref(),
            );
        }

        Err(anyhow!(
            "未检测到数据库连接配置，请在 config/default.toml 或环境变量中设置 `database.*` 或 `spring.datasource.*`"
        ))
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseSettings {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "driver-class-name")]
    pub driver_class_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
    pub token_ttl_minutes: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExecutorSettings {
    pub access_token: Option<String>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

impl ExecutorSettings {
    pub fn access_token(&self) -> Option<&str> {
        self.access_token
            .as_deref()
            .map(|token| token.trim())
            .filter(|token| !token.is_empty())
    }

    pub fn timeout_seconds(&self) -> u64 {
        let raw = self.timeout_seconds.unwrap_or(3);
        raw.clamp(1, 10)
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SpringSettings {
    #[serde(default)]
    pub datasource: SpringDatasourceSettings,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SpringDatasourceSettings {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "driver-class-name")]
    pub driver_class_name: Option<String>,
}

impl SpringDatasourceSettings {
    fn apply_env_overrides(&mut self) {
        if let Ok(url) = std::env::var("SPRING_DATASOURCE_URL") {
            if !url.trim().is_empty() {
                self.url = Some(url);
            }
        }

        if let Ok(username) = std::env::var("SPRING_DATASOURCE_USERNAME") {
            if !username.trim().is_empty() {
                self.username = Some(username);
            }
        }

        if let Ok(password) = std::env::var("SPRING_DATASOURCE_PASSWORD") {
            if !password.trim().is_empty() {
                self.password = Some(password);
            }
        }

        if let Ok(driver) = std::env::var("SPRING_DATASOURCE_DRIVER_CLASS_NAME") {
            if !driver.trim().is_empty() {
                self.driver_class_name = Some(driver);
            }
        }
    }
}

fn build_mysql_url(
    raw_url: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> anyhow::Result<String> {
    let trimmed = raw_url.trim();
    let without_jdbc = trimmed.strip_prefix("jdbc:").unwrap_or(trimmed);

    let mut url =
        Url::parse(without_jdbc).with_context(|| format!("解析数据库连接 URL 失败: {raw_url}"))?;

    if let Some(user) = username {
        url.set_username(user)
            .map_err(|_| anyhow!("设置数据库用户名失败"))?;
    }

    if let Some(pass) = password {
        url.set_password(Some(pass))
            .map_err(|_| anyhow!("设置数据库密码失败"))?;
    }

    Ok(url.to_string())
}
