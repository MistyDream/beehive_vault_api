use std::sync::Arc;

use chrono::NaiveDate;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::application::services::pagination::{paginate_slice, Page, DEFAULT_LIMIT, DEFAULT_PAGE};
use crate::application::services::stock_lookup::fetch_stocks_for_transactions;
use crate::domain::wallet::cash_balance::{compute_cash_balance, CashBalance};
use crate::domain::wallet::performance::{compute_performance, PerformanceReport};
use crate::domain::wallet::portfolio_summary::PortfolioSummary;
use crate::domain::wallet::position::{compute_positions, Position};
use crate::domain::wallet::transaction::TransactionFilter;

#[derive(Debug, Default, Clone)]
pub struct PositionsQuery {
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

pub struct PositionService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    stock_repo: Arc<dyn StockRepository>,
}

impl PositionService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        stock_repo: Arc<dyn StockRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo, stock_repo }
    }

    pub async fn get_positions(&self, portfolio_id: i32) -> Result<Vec<Position>, AppError> {
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &transactions).await?;
        Ok(compute_positions(&transactions, &stocks))
    }

    pub async fn get_positions_paginated(
        &self,
        portfolio_id: i32,
        query: PositionsQuery,
    ) -> Result<Page<Position>, AppError> {
        let mut positions = self.get_positions(portfolio_id).await?;

        let ascending = query.sort_dir.as_deref() == Some("asc");
        let tiebreaker = |a: &Position, b: &Position| a.stock.id.cmp(&b.stock.id);

        positions.sort_by(|a, b| {
            let ord = match query.sort_by.as_deref().unwrap_or("weight") {
                "symbol" => a.stock.symbol.cmp(&b.stock.symbol),
                "quantity" => a.quantity.partial_cmp(&b.quantity).unwrap_or(std::cmp::Ordering::Equal),
                "average_cost" => a.average_cost.partial_cmp(&b.average_cost).unwrap_or(std::cmp::Ordering::Equal),
                "total_cost" => a.total_cost.partial_cmp(&b.total_cost).unwrap_or(std::cmp::Ordering::Equal),
                _ => a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal),
            };
            let ord = if ascending { ord } else { ord.reverse() };
            ord.then_with(|| tiebreaker(a, b))
        });

        let page = query.page.unwrap_or(DEFAULT_PAGE);
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
        Ok(paginate_slice(positions, page, limit))
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
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &transactions).await?;
        let positions = compute_positions(&transactions, &stocks);
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
                transaction_types: Vec::new(),
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
