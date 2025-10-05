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
}

impl AppState {
    pub fn new(db: DatabaseConnection, settings: Settings) -> Self {
        Self {
            inner: Arc::new(AppStateInner { db, settings }),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn settings(&self) -> &Settings {
        &self.inner.settings
    }
}
