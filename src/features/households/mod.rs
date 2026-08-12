mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{Router, routing::post};

use crate::database::Database;

use repository::HouseholdRepository;
use service::HouseholdService;

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
        .route("/households", post(handlers::create))
        .route(
            "/households/{household_id}",
            axum::routing::get(handlers::get),
        )
        .with_state(module)
}
