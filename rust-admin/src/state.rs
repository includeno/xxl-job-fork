use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub db: DatabaseConnection,
    pub settings: Settings,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(db: DatabaseConnection, settings: Settings, http_client: reqwest::Client) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                settings,
                http_client,
            }),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn settings(&self) -> &Settings {
        &self.inner.settings
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.inner.http_client
    }
}
