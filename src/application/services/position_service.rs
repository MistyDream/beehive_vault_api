use std::sync::Arc;

use chrono::NaiveDate;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::domain::wallet::cash_balance::{compute_cash_balance, CashBalance};
use crate::domain::wallet::performance::{compute_performance, PerformanceReport};
use crate::domain::wallet::portfolio_summary::PortfolioSummary;
use crate::domain::wallet::position::{compute_positions, Position};
use crate::domain::wallet::transaction::TransactionFilter;

pub struct PositionService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl PositionService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo }
    }

    pub async fn get_positions(&self, portfolio_id: i32) -> Result<Vec<Position>, AppError> {
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        Ok(compute_positions(&transactions))
    }

    pub async fn get_cash_balance(&self, portfolio_id: i32) -> Result<CashBalance, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        Ok(compute_cash_balance(&transactions, &portfolio.currency))
    }

    pub async fn get_summary(&self, portfolio_id: i32) -> Result<PortfolioSummary, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        let positions = compute_positions(&transactions);
        let cash = compute_cash_balance(&transactions, &portfolio.currency);
        let total_invested: f64 = positions.iter().map(|p| p.total_cost).sum();
        Ok(PortfolioSummary { portfolio, positions, cash, total_invested })
    }

    pub async fn get_performance(
        &self,
        portfolio_id: i32,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Result<PerformanceReport, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = if from_date.is_some() || to_date.is_some() {
            let filters = TransactionFilter {
                transaction_type: None,
                stock_id: None,
                from_date,
                to_date,
            };
            self.transaction_repo
                .list_by_portfolio_filtered(portfolio_id, filters)
                .await?
        } else {
            self.transaction_repo
                .list_by_portfolio_chronological(portfolio_id)
                .await?
        };
        Ok(compute_performance(portfolio_id, &portfolio.currency, &transactions))
    }
}
