use anyhow::Result;
use dotenv::dotenv;

use beehive_vault_api::infrastructure::http::server;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    server::run().await
}
