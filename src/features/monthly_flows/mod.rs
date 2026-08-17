mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{Router, routing::get};

use crate::{database::Database, features::households::HouseholdRepository};

use repository::MonthlyFlowRepository;
use service::MonthlyFlowService;

#[derive(Clone)]
pub(crate) struct MonthlyFlowsModule {
    service: MonthlyFlowService,
}

pub(crate) fn configure(database: Database) -> MonthlyFlowsModule {
    let repository = MonthlyFlowRepository::new(database.clone());
    let household_repository = HouseholdRepository::new(database);
    let service = MonthlyFlowService::new(repository, household_repository);

    MonthlyFlowsModule { service }
}

pub(crate) fn routes(module: MonthlyFlowsModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/monthly-flows/{month}",
            get(handlers::get),
        )
        .with_state(module)
}
