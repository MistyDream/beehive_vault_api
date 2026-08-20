mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{Router, routing::get};

use crate::database::Database;

use service::HouseholdService;

pub(crate) use repository::HouseholdRepository;

#[derive(Clone)]
pub(crate) struct HouseholdsModule {
    service: HouseholdService,
}

pub(crate) fn configure(database: Database) -> HouseholdsModule {
    let repository = HouseholdRepository::new(database);
    let service = HouseholdService::new(repository);

    HouseholdsModule { service }
}

pub(crate) fn routes(module: HouseholdsModule) -> Router {
    Router::new()
        .route("/households", get(handlers::list).post(handlers::create))
        .route("/households/{household_id}", get(handlers::get))
        .with_state(module)
}
