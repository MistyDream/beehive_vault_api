//! Integration-test scaffolding: in-memory or no-op port implementations and a
//! helper to assemble a full `AppState` without hitting any real database or
//! external service.

pub mod fakes;

/// Builds an actix test service mirroring the production app: bearer auth is
/// dropped (the `/v1` scope is mounted bare) but the `QueryConfig` error
/// handler is registered so garde validation failures surface as 422 +
/// problem+json instead of garde-actix-web's default 400.
#[macro_export]
macro_rules! make_service {
    ($stock_repo:expr, $price_repo:expr $(,)?) => {{
        let state = $crate::common::build_app_state($stock_repo, $price_repo);
        ::actix_web::test::init_service(
            ::actix_web::App::new()
                .app_data(::actix_web::web::Data::new(state))
                .app_data(
                    ::garde_actix_web::web::QueryConfig::default().error_handler(
                        ::beehive_vault_api::infrastructure::http::error::garde_error_handler,
                    ),
                )
                .app_data(
                    ::garde_actix_web::web::JsonConfig::default().error_handler(
                        ::beehive_vault_api::infrastructure::http::error::garde_error_handler,
                    ),
                )
                .app_data(
                    ::actix_web::web::PathConfig::default().error_handler(
                        ::beehive_vault_api::infrastructure::http::error::path_error_handler,
                    ),
                )
                .service(::actix_web::web::scope("/v1").configure(
                    ::beehive_vault_api::infrastructure::http::routes::configure_routes,
                )),
        )
        .await
    }};
}

use std::sync::Arc;

use beehive_vault_api::application::ports::stock_price_repository::StockPriceRepository;
use beehive_vault_api::application::ports::stock_repository::StockRepository;
use beehive_vault_api::application::services::portfolio_scoring_service::PortfolioScoringService;
use beehive_vault_api::application::services::portfolio_service::PortfolioService;
use beehive_vault_api::application::services::position_service::PositionService;
use beehive_vault_api::application::services::price_service::PriceService;
use beehive_vault_api::application::services::stock_service::StockService;
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
    let price_service = Arc::new(PriceService::new(stock_repo.clone(), stock_price_repo));
    let stock_service = Arc::new(StockService::new(stock_repo));
    let health_checker = Arc::new(AlwaysReadyHealthChecker);

    AppState {
        portfolio_service,
        transaction_service,
        position_service,
        portfolio_scoring_service,
        price_service,
        stock_service,
        health_checker,
    }
}
