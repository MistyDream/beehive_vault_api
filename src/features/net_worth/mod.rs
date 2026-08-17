mod domain;
mod handlers;
mod service;

use axum::{Router, routing::get};

use crate::{database::Database, features::accounts::AccountRepository};

use service::NetWorthService;

#[derive(Clone)]
pub(crate) struct NetWorthModule {
    service: NetWorthService,
}

pub(crate) fn configure(database: Database) -> NetWorthModule {
    let account_repository = AccountRepository::new(database);
    let service = NetWorthService::new(account_repository);

    NetWorthModule { service }
}

pub(crate) fn routes(module: NetWorthModule) -> Router {
    Router::new()
        .route("/households/{household_id}/summary", get(handlers::summary))
        .with_state(module)
}
