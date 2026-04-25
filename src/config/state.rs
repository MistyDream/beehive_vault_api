use std::sync::Arc;

use anyhow::Result;

use crate::application::ports::health_checker::HealthChecker;
use crate::application::services::portfolio_scoring_service::PortfolioScoringService;
use crate::application::services::portfolio_service::PortfolioService;
use crate::application::services::position_service::PositionService;
use crate::application::services::price_batch_service::PriceBatchService;
use crate::application::services::price_service::PriceService;
use crate::application::services::transaction_service::TransactionService;
use crate::config::settings;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::market::yfinance_price_fetcher::YFinancePriceFetcher;
use crate::infrastructure::persistence::connect;
use crate::infrastructure::persistence::repositories::portfolio_repository::PgPortfolioRepository;
use crate::infrastructure::persistence::repositories::score_snapshot_repository::PgScoreSnapshotRepository;
use crate::infrastructure::persistence::repositories::stock_price_repository::PgStockPriceRepository;
use crate::infrastructure::persistence::repositories::stock_repository::PgStockRepository;
use crate::infrastructure::persistence::repositories::transaction_repository::PgTransactionRepository;

/// Services assembled at startup. `http` feeds the Actix handlers;
/// `price_batch` is held separately because it only has a background-job
/// consumer (the cron scheduler).
pub struct Services {
    pub http: AppState,
    pub price_batch: Arc<PriceBatchService>,
}

pub fn init() -> Result<Services> {
    let config = settings::get();
    let db = connect(&config.db)?;

    let portfolio_repo = Arc::new(PgPortfolioRepository::new(db.clone()));
    let transaction_repo = Arc::new(PgTransactionRepository::new(db.clone()));
    let stock_repo = Arc::new(PgStockRepository::new(db.clone()));
    let stock_price_repo = Arc::new(PgStockPriceRepository::new(db.clone()));
    let score_repo = Arc::new(PgScoreSnapshotRepository::new(db.clone()));
    let health_checker = Arc::new(db) as Arc<dyn HealthChecker>;
    let price_fetcher = Arc::new(YFinancePriceFetcher::new());

    let portfolio_service = Arc::new(PortfolioService::new(portfolio_repo.clone()));
    let transaction_service = Arc::new(TransactionService::new(
        portfolio_repo.clone(),
        transaction_repo.clone(),
        stock_repo.clone(),
    ));
    let position_service = Arc::new(PositionService::new(
        portfolio_repo.clone(),
        transaction_repo.clone(),
        stock_repo.clone(),
        stock_price_repo.clone(),
    ));
    let portfolio_scoring_service = Arc::new(PortfolioScoringService::new(
        portfolio_repo,
        transaction_repo,
        stock_repo.clone(),
        score_repo,
    ));
    let price_service = Arc::new(PriceService::new(stock_repo.clone(), stock_price_repo.clone()));
    let price_batch_service = Arc::new(PriceBatchService::new(
        stock_repo,
        stock_price_repo,
        price_fetcher,
    ));

    Ok(Services {
        http: AppState {
            portfolio_service,
            transaction_service,
            position_service,
            portfolio_scoring_service,
            price_service,
            health_checker,
        },
        price_batch: price_batch_service,
    })
}
