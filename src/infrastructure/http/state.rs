use std::sync::Arc;

use crate::application::ports::health_checker::HealthChecker;
use crate::application::services::portfolio_scoring_service::PortfolioScoringService;
use crate::application::services::portfolio_service::PortfolioService;
use crate::application::services::position_service::PositionService;
use crate::application::services::price_service::PriceService;
use crate::application::services::stock_service::StockService;
use crate::application::services::transaction_service::TransactionService;

#[derive(Clone)]
pub struct AppState {
    pub portfolio_service: Arc<PortfolioService>,
    pub transaction_service: Arc<TransactionService>,
    pub position_service: Arc<PositionService>,
    pub portfolio_scoring_service: Arc<PortfolioScoringService>,
    pub price_service: Arc<PriceService>,
    pub stock_service: Arc<StockService>,
    pub health_checker: Arc<dyn HealthChecker>,
}
