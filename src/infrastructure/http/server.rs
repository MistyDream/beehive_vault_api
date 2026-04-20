use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::{App, HttpServer, web, http};
use anyhow::Result;
use garde_actix_web::web::{JsonConfig, QueryConfig};

use crate::config::settings;
use crate::infrastructure::http::error::garde_error_handler;
use crate::infrastructure::http::request_context::REQUEST_PATH;
use crate::infrastructure::http::routes::configure_routes;
use crate::infrastructure::http::state::AppState;

pub async fn run(state: AppState) -> Result<()> {
    let server_config = &settings::get().server;
    let cors_config = settings::get().cors.clone();
    let app_state = web::Data::new(state);

    let app = move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::ACCEPT])
            .supports_credentials()
            .max_age(3600);

        for origin in &cors_config.allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(app_state.clone())
            .app_data(JsonConfig::default().error_handler(garde_error_handler))
            .app_data(QueryConfig::default().error_handler(garde_error_handler))
            .wrap_fn(|req, srv| {
                let path = req.path().to_string();
                REQUEST_PATH.scope(path, srv.call(req))
            })
            .wrap(cors)
            .configure(configure_routes)
    };

    HttpServer::new(app)
        .bind((server_config.host.as_str(), server_config.port))?
        .run()
        .await?;

    Ok(())
}
