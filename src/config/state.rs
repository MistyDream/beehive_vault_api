use std::sync::Arc;

use anyhow::Result;

use crate::application::services::portfolio_scoring_service::PortfolioScoringService;
use crate::application::services::portfolio_service::PortfolioService;
use crate::application::services::position_service::PositionService;
use crate::application::services::transaction_service::TransactionService;
use crate::config::settings;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::connect;
use crate::infrastructure::persistence::repositories::portfolio_repository::PgPortfolioRepository;
use crate::infrastructure::persistence::repositories::score_snapshot_repository::PgScoreSnapshotRepository;
use crate::infrastructure::persistence::repositories::stock_repository::PgStockRepository;
use crate::infrastructure::persistence::repositories::transaction_repository::PgTransactionRepository;

pub fn init() -> Result<AppState> {
    let config = settings::get();
    let db = connect(&config.db)?;

    let portfolio_repo = Arc::new(PgPortfolioRepository::new(db.clone()));
    let transaction_repo = Arc::new(PgTransactionRepository::new(db.clone()));
    let stock_repo = Arc::new(PgStockRepository::new(db.clone()));
    let score_repo = Arc::new(PgScoreSnapshotRepository::new(db));

    let portfolio_service = Arc::new(PortfolioService::new(portfolio_repo.clone()));
    let transaction_service = Arc::new(TransactionService::new(
        portfolio_repo.clone(),
        transaction_repo.clone(),
    ));
    let position_service = Arc::new(PositionService::new(
        portfolio_repo.clone(),
        transaction_repo.clone(),
    ));
    let portfolio_scoring_service = Arc::new(PortfolioScoringService::new(
        portfolio_repo,
        transaction_repo,
        stock_repo,
        score_repo,
    ));

    Ok(AppState {
        portfolio_service,
        transaction_service,
        position_service,
        portfolio_scoring_service,
    })
}
