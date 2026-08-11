mod handlers;

use axum::{Router, routing::get};

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct NetWorthModule {
    database: Database,
}

pub(crate) fn configure(database: Database) -> NetWorthModule {
    NetWorthModule { database }
}

pub(crate) fn routes(module: NetWorthModule) -> Router {
    Router::new()
        .route("/households/{household_id}/summary", get(handlers::summary))
        .with_state(module)
}
