use anyhow::Result;
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

use beehive_vault_api::config::{settings, state};
use beehive_vault_api::infrastructure::http::server;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    settings::init()?;
    let state = state::init()?;

    server::run(state).await
}
