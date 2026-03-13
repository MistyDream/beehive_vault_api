use anyhow::Result;
use dotenvy::dotenv;

use beehive_vault_api::config::{settings, state};
use beehive_vault_api::infrastructure::http::server;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    settings::init()?;
    let state = state::init()?;

    server::run(state).await
}
