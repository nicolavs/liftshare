use crate::settings::SETTINGS;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(SETTINGS.database.url.as_str())
        .await
        .expect("can't connect to database")
}
