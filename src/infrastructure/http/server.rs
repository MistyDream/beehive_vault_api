use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::dev::Service;
use actix_web::middleware::DefaultHeaders;
use actix_web::{App, HttpServer, web, http};
use anyhow::Result;
use garde_actix_web::web::{JsonConfig, QueryConfig};
use tracing_actix_web::TracingLogger;

use crate::config::settings;
use crate::infrastructure::http::controllers::health_controller;
use crate::infrastructure::http::error::{garde_error_handler, path_error_handler};
use crate::infrastructure::http::middleware::auth::BearerAuth;
use crate::infrastructure::http::request_context::REQUEST_PATH;
use crate::infrastructure::http::routes::configure_routes;
use crate::infrastructure::http::state::AppState;

const JSON_BODY_LIMIT: usize = 32 * 1024;

pub async fn run(state: AppState) -> Result<()> {
    let server_config = &settings::get().server;
    let cors_config = settings::get().cors.clone();
    let api_key = settings::get().auth.api_key.clone();
    let app_state = web::Data::new(state);

    // Per-worker limit: actix-governor does not share state across workers,
    // so effective global ceiling is `workers × requests_per_second`.
    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_second(60)
        .burst_size(30)
        .finish()
        .expect("invalid governor config");

    let app = move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::ACCEPT, http::header::IF_NONE_MATCH])
            .expose_headers(vec![http::header::LOCATION, http::header::ETAG])
            .supports_credentials()
            .max_age(3600);

        for origin in &cors_config.allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        let security_headers = DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Frame-Options", "DENY"))
            .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
            .add(("Strict-Transport-Security", "max-age=63072000; includeSubDomains"));

        App::new()
            .app_data(app_state.clone())
            .app_data(
                JsonConfig::default()
                    .limit(JSON_BODY_LIMIT)
                    .error_handler(garde_error_handler),
            )
            .app_data(QueryConfig::default().error_handler(garde_error_handler))
            .app_data(web::PathConfig::default().error_handler(path_error_handler))
            .wrap_fn(|req, srv| {
                let path = req.path().to_string();
                REQUEST_PATH.scope(path, srv.call(req))
            })
            .wrap(security_headers)
            .wrap(cors)
            .wrap(TracingLogger::default())
            .configure(health_controller::configure)
            .service(
                web::scope("/v1")
                    .wrap(BearerAuth::new(api_key.clone()))
                    .wrap(Governor::new(&governor_conf))
                    .configure(configure_routes),
            )
    };

    HttpServer::new(app)
        .bind((server_config.host.as_str(), server_config.port))?
        .run()
        .await?;

    Ok(())
}
