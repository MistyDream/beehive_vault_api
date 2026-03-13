use std::sync::Arc;

use anyhow::Result;

use crate::application::services::stock_service::StockService;
use crate::config::settings;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::connect;
use crate::infrastructure::persistence::repositories::stock_repository::PgStockRepository;

pub fn init() -> Result<AppState> {
    let config = settings::get();
    let db = connect(&config.db)?;

    let stock_repo = Arc::new(PgStockRepository::new(db));
    let stock_service = Arc::new(StockService::new(stock_repo));

    Ok(AppState { stock_service })
}
