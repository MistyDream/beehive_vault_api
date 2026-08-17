pub mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    database::Database,
    features::{
        accounts::AccountRepository, categories::CategoryRepository,
        households::HouseholdRepository,
    },
};

use repository::TransactionRepository;
use service::TransactionService;

#[derive(Clone)]
pub(crate) struct TransactionsModule {
    service: TransactionService,
}

pub(crate) fn configure(database: Database) -> TransactionsModule {
    let transaction_repository = TransactionRepository::new(database.clone());
    let household_repository = HouseholdRepository::new(database.clone());
    let account_repository = AccountRepository::new(database.clone());
    let category_repository = CategoryRepository::new(database);
    let service = TransactionService::new(
        transaction_repository,
        household_repository,
        account_repository,
        category_repository,
    );

    TransactionsModule { service }
}

pub(crate) fn routes(module: TransactionsModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/transactions",
            post(handlers::create).get(handlers::list),
        )
        .route(
            "/households/{household_id}/transactions/{transaction_id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .with_state(module)
}
