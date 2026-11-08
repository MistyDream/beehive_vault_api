mod domain;
mod dto;
mod handlers;
mod reader;
mod repository;
mod service;

use axum::{Router, extract::DefaultBodyLimit, routing::post};

use crate::{
    database::Database,
    features::{accounts::AccountRepository, households::HouseholdRepository},
};

use repository::CsvImportRepository;
use service::CsvImportService;

// Keep the preview's 2 MiB allowance for the encoded CSV and format. Another 512 KiB
// leaves room for the file name and 5,000 selections of fewer than 60 bytes each in compact JSON.
const MAX_CONFIRM_BODY_BYTES: usize = 2 * 1024 * 1024 + 512 * 1024;

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
            post(handlers::confirm).layer(DefaultBodyLimit::max(MAX_CONFIRM_BODY_BYTES)),
        )
        .with_state(module)
}
