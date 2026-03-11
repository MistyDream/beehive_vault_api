use anyhow::Result;
use dotenvy::dotenv;

use beehive_vault_api::config::settings;
use beehive_vault_api::infrastructure::http::server;
use beehive_vault_api::infrastructure::http::state::AppState;
use beehive_vault_api::infrastructure::persistence::connect;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let config = settings::init()?;
    let db = connect(&config.db)?;
    let state = AppState::new(db);

    server::run(state).await
}
