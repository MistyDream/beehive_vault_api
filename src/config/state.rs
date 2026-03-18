use std::sync::Arc;

use anyhow::Result;

use crate::application::services::gurufocus_service::GurufocusService;
use crate::application::services::stock_service::StockService;
use crate::config::settings;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::connect;
use crate::infrastructure::persistence::repositories::metric_value_repository::PgMetricValueRepository;
use crate::infrastructure::persistence::repositories::score_repository::PgScoreRepository;
use crate::infrastructure::persistence::repositories::stock_repository::PgStockRepository;

pub fn init() -> Result<AppState> {
    let config = settings::get();
    let db = connect(&config.db)?;

    let stock_repo = Arc::new(PgStockRepository::new(db.clone()));
    let metric_value_repo = Arc::new(PgMetricValueRepository::new(db.clone()));
    let score_repo = Arc::new(PgScoreRepository::new(db));

    let stock_service = Arc::new(StockService::new(stock_repo.clone()));
    let gurufocus_service = Arc::new(GurufocusService::new(metric_value_repo, score_repo));

    Ok(AppState { stock_service, gurufocus_service })
}
