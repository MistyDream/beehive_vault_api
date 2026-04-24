use anyhow::Result;
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

use beehive_vault_api::config::{settings, state};
use beehive_vault_api::infrastructure::http::server;
use beehive_vault_api::infrastructure::scheduler;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    settings::init()?;
    let services = state::init()?;
    let _scheduler = scheduler::start(services.price_batch).await?;

    server::run(services.http).await
}
