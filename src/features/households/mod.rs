mod handlers;

use axum::{Router, routing::post};

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct HouseholdsModule {
    database: Database,
}

pub(crate) fn configure(database: Database) -> HouseholdsModule {
    HouseholdsModule { database }
}

pub(crate) fn routes(module: HouseholdsModule) -> Router {
    Router::new()
        .route("/households", post(handlers::create))
        .route(
            "/households/{household_id}",
            axum::routing::get(handlers::get),
        )
        .with_state(module)
}
