pub mod admin;
mod handlers;

use axum::{Router, routing::get};

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct InstitutionsModule {
    database: Database,
}

pub(crate) fn configure(database: Database) -> InstitutionsModule {
    InstitutionsModule { database }
}

pub(crate) fn routes(module: InstitutionsModule) -> Router {
    Router::new()
        .route("/institutions", get(handlers::list))
        .with_state(module)
}
