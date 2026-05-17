use std::sync::Arc;
pub type SharedState = Arc<AppState>;

use sqlx::postgres::PgPool;
pub struct AppState {
    pub db_pool: PgPool,
}
