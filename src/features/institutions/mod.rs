mod handlers;

use axum::{
    Router,
    routing::{patch, post},
};

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
        .route(
            "/households/{household_id}/institutions",
            post(handlers::create).get(handlers::list),
        )
        .route(
            "/households/{household_id}/institutions/{institution_id}",
            patch(handlers::update).delete(handlers::archive),
        )
        .with_state(module)
}
