mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::database::Database;

use service::AccountService;

pub(crate) use domain::{Account, AccountKind};
pub(crate) use repository::AccountRepository;

#[derive(Clone)]
pub(crate) struct AccountsModule {
    service: AccountService,
}

pub(crate) fn configure(database: Database) -> AccountsModule {
    let repository = AccountRepository::new(database);
    let service = AccountService::new(repository);

    AccountsModule { service }
}

pub(crate) fn routes(module: AccountsModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/accounts",
            post(handlers::create).get(handlers::list),
        )
        .route(
            "/households/{household_id}/accounts/{account_id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::archive),
        )
        .route(
            "/households/{household_id}/accounts/{account_id}/balances",
            post(handlers::create_balance).get(handlers::list_balances),
        )
        .route(
            "/households/{household_id}/accounts/{account_id}/balances/{balance_id}",
            patch(handlers::update_balance),
        )
        .with_state(module)
}
