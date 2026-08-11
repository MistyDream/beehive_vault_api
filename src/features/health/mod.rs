mod handlers;

use axum::{Router, routing::get};

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct HealthModule {
    database: Database,
}

pub(crate) fn configure(database: Database) -> HealthModule {
    HealthModule { database }
}

pub(crate) fn routes(module: HealthModule) -> Router {
    Router::new()
        .route("/healthz", get(handlers::liveness))
        .route("/readyz", get(handlers::readiness))
        .with_state(module)
}
