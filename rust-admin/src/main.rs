mod auth;
mod config;
mod entities;
mod error;
mod request_preview;
mod routes;
mod state;
mod templates;

use std::net::SocketAddr;

use axum::serve;
use dotenvy::dotenv;
use sea_orm::Database;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Settings;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    setup_tracing();

    let settings = Settings::load()?;
    let addr: SocketAddr = settings.server.socket_addr()?;
    let database_url = settings.database_url()?;
    let db = Database::connect(&database_url).await?;
    let http_client = build_http_client(&settings)?;

    let state = AppState::new(db, settings, http_client);
    let app = routes::create_router(state);

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Rust admin 服务启动: {}", listener.local_addr()?);
    serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn setup_tracing() {
    let env_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "rust_admin=info,axum::rejection=trace".into());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(env_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_http_client(settings: &Settings) -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(settings.executor.timeout())
        // 默认的系统代理会把内网地址转发给代理服务器，导致执行器连接被拒绝。
        // 这里显式关闭代理检测，确保所有触发请求都直接访问执行器。
        .no_proxy()
        .build()?)
}
