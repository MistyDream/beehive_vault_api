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
    features::{accounts::AccountRepository, households::HouseholdRepository},
};

use repository::TransferRepository;
use service::TransferService;

#[derive(Clone)]
pub(crate) struct TransfersModule {
    service: TransferService,
}

pub(crate) fn configure(database: Database) -> TransfersModule {
    let transfer_repository = TransferRepository::new(database.clone());
    let household_repository = HouseholdRepository::new(database.clone());
    let account_repository = AccountRepository::new(database);
    let service = TransferService::new(
        transfer_repository,
        household_repository,
        account_repository,
    );

    TransfersModule { service }
}

pub(crate) fn routes(module: TransfersModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/transfers",
            post(handlers::create).get(handlers::list),
        )
        .route(
            "/households/{household_id}/transfers/{transfer_id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .with_state(module)
}
