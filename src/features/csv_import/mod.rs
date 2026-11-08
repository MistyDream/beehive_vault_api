mod domain;
mod dto;
mod handlers;
mod reader;
mod repository;
mod service;

use axum::{Router, routing::post};

use crate::{
    database::Database,
    features::{accounts::AccountRepository, households::HouseholdRepository},
};

use repository::CsvImportRepository;
use service::CsvImportService;

#[derive(Clone)]
pub(crate) struct CsvImportModule {
    service: CsvImportService,
}

pub(crate) fn configure(database: Database) -> CsvImportModule {
    let accounts = AccountRepository::new(database.clone());
    let households = HouseholdRepository::new(database.clone());
    let imports = CsvImportRepository::new(database);
    let service = CsvImportService::new(accounts, households, imports);

    CsvImportModule { service }
}

pub(crate) fn routes(module: CsvImportModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/accounts/{account_id}/imports/csv/preview",
            post(handlers::preview),
        )
        .route(
            "/households/{household_id}/accounts/{account_id}/imports/csv/confirm",
            post(handlers::confirm),
        )
        .with_state(module)
}
