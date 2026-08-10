pub mod app;
pub mod config;
pub mod error;
pub mod features;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
