use actix_web::{App, HttpServer, web};
use anyhow::Result;

use crate::config::settings;
use crate::infrastructure::http::routes::configure_routes;
use crate::infrastructure::http::state::AppState;

pub async fn run(state: AppState) -> Result<()> {
    let server_config = &settings::get().server;
    let app_state = web::Data::new(state);

    let app = move || {
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes)
    };

    HttpServer::new(app)
        .bind((server_config.host.as_str(), server_config.port))?
        .run()
        .await?;

    Ok(())
}
