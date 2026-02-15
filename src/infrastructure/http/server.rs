use actix_web::{App, HttpServer};
use anyhow::{Error, Result};

use crate::config::settings;

use crate::infrastructure::http::routes::configure_routes;

pub async fn run() -> Result<()> {
    let server = settings::load_config()?.server;

    let app = move || {
        App::new()
            .configure(configure_routes)
    };

    HttpServer::new(app)
    .bind((server.host, server.port))?
    .run()
    .await
    .map_err(Error::from)
}

// mod routes {
//     use actix_web::{web, HttpResponse};

//     pub fn init(cfg: &mut web::ServiceConfig) {
//         cfg.route("/", web::get().to(health_check));
//     }

//     async fn health_check() -> HttpResponse {
//         HttpResponse::Ok().json("Server is running!")
//     }
// }