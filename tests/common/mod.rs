//! Integration-test scaffolding: in-memory or no-op port implementations and a
//! helper to assemble a full `AppState` without hitting any real database or
//! external service.

pub mod fakes;

use std::sync::Arc;

use beehive_vault_api::application::ports::stock_price_repository::StockPriceRepository;
use beehive_vault_api::application::ports::stock_repository::StockRepository;
use beehive_vault_api::application::services::portfolio_scoring_service::PortfolioScoringService;
use beehive_vault_api::application::services::portfolio_service::PortfolioService;
use beehive_vault_api::application::services::position_service::PositionService;
use beehive_vault_api::application::services::price_service::PriceService;
use beehive_vault_api::application::services::transaction_service::TransactionService;
use beehive_vault_api::infrastructure::http::state::AppState;

use fakes::{
    AlwaysReadyHealthChecker, NoOpPortfolioRepo, NoOpScoreSnapshotRepo, NoOpTransactionRepo,
};

/// Build a complete `AppState` where only the stock + price repositories are
/// wired with in-memory fakes — all other repos are no-ops since the routes
/// under test never reach them.
pub fn build_app_state(
    stock_repo: Arc<dyn StockRepository>,
    stock_price_repo: Arc<dyn StockPriceRepository>,
) -> AppState {
    let portfolio_repo = Arc::new(NoOpPortfolioRepo);
    let transaction_repo = Arc::new(NoOpTransactionRepo);
    let score_repo = Arc::new(NoOpScoreSnapshotRepo);

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
    let price_service = Arc::new(PriceService::new(stock_repo, stock_price_repo));
    let health_checker = Arc::new(AlwaysReadyHealthChecker);

    AppState {
        portfolio_service,
        transaction_service,
        position_service,
        portfolio_scoring_service,
        price_service,
        health_checker,
    }
}
