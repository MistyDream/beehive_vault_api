use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::stock_price_repository::StockPriceRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::application::services::pagination::{paginate_slice, Page, SortDirection, DEFAULT_LIMIT, DEFAULT_PAGE};
use crate::application::services::stock_lookup::fetch_stocks_for_transactions;
use crate::domain::wallet::cash_balance::{compute_cash_balance, CashBalance};
use crate::domain::wallet::performance::{compute_performance, PerformanceReport};
use crate::domain::wallet::portfolio_summary::PortfolioSummary;
use crate::domain::wallet::position::{compute_positions, valorize_positions, Position};
use crate::domain::wallet::transaction::TransactionFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionSort {
    #[default]
    Weight,
    Symbol,
    Quantity,
    AverageCost,
    TotalCost,
}

impl PositionSort {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("symbol") => PositionSort::Symbol,
            Some("quantity") => PositionSort::Quantity,
            Some("average_cost") => PositionSort::AverageCost,
            Some("total_cost") => PositionSort::TotalCost,
            _ => PositionSort::Weight,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PositionsQuery {
    pub sort_by: PositionSort,
    pub sort_dir: SortDirection,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

pub struct PositionService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    stock_repo: Arc<dyn StockRepository>,
    price_repo: Arc<dyn StockPriceRepository>,
}

impl PositionService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        stock_repo: Arc<dyn StockRepository>,
        price_repo: Arc<dyn StockPriceRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo, stock_repo, price_repo }
    }

    pub async fn get_positions(&self, portfolio_id: Uuid) -> Result<Vec<Position>, AppError> {
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &transactions).await?;
        let mut positions = compute_positions(&transactions, &stocks);
        self.enrich_with_latest_prices(&mut positions).await?;
        Ok(positions)
    }

    async fn enrich_with_latest_prices(&self, positions: &mut [Position]) -> Result<(), AppError> {
        if positions.is_empty() {
            return Ok(());
        }
        let stock_ids: Vec<i32> = positions.iter().map(|p| p.stock.id).collect();
        let prices = self.price_repo.find_latest_batch(stock_ids).await?;
        valorize_positions(positions, &prices);
        Ok(())
    }

    pub async fn get_positions_paginated(
        &self,
        portfolio_id: Uuid,
        query: PositionsQuery,
    ) -> Result<Page<Position>, AppError> {
        let mut positions = self.get_positions(portfolio_id).await?;

        let ascending = query.sort_dir == SortDirection::Asc;
        let tiebreaker = |a: &Position, b: &Position| a.stock.id.cmp(&b.stock.id);

        positions.sort_by(|a, b| {
            let ord = match query.sort_by {
                PositionSort::Symbol => a.stock.symbol.cmp(&b.stock.symbol),
                PositionSort::Quantity => a.quantity.partial_cmp(&b.quantity).unwrap_or(std::cmp::Ordering::Equal),
                PositionSort::AverageCost => a.average_cost.partial_cmp(&b.average_cost).unwrap_or(std::cmp::Ordering::Equal),
                PositionSort::TotalCost => a.total_cost.partial_cmp(&b.total_cost).unwrap_or(std::cmp::Ordering::Equal),
                PositionSort::Weight => a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal),
            };
            let ord = if ascending { ord } else { ord.reverse() };
            ord.then_with(|| tiebreaker(a, b))
        });

        let page = query.page.unwrap_or(DEFAULT_PAGE);
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
        Ok(paginate_slice(positions, page, limit))
    }

    pub async fn get_cash_balance(&self, portfolio_id: Uuid) -> Result<CashBalance, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        Ok(compute_cash_balance(&transactions, &portfolio.currency))
    }

    pub async fn get_summary(&self, portfolio_id: Uuid) -> Result<PortfolioSummary, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &transactions).await?;
        let mut positions = compute_positions(&transactions, &stocks);
        self.enrich_with_latest_prices(&mut positions).await?;
        let cash = compute_cash_balance(&transactions, &portfolio.currency);

        let total_invested: f64 = positions.iter().map(|p| p.total_cost).sum();
        let positions_without_price = positions
            .iter()
            .filter(|p| p.current_value.is_none())
            .count();
        let total_value = cash.balance
            + positions
                .iter()
                .filter_map(|p| p.current_value)
                .sum::<f64>();

        Ok(PortfolioSummary {
            portfolio,
            positions,
            cash,
            total_invested,
            total_value,
            positions_without_price,
        })
    }

    pub async fn get_performance(
        &self,
        portfolio_id: Uuid,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Result<PerformanceReport, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = if from_date.is_some() || to_date.is_some() {
            let filters = TransactionFilter {
                from_date,
                to_date,
                ..Default::default()
            };
            self.transaction_repo
                .list_by_portfolio_filtered(portfolio_id, filters)
                .await?
        } else {
            self.transaction_repo
                .list_by_portfolio_chronological(portfolio_id)
                .await?
        };

        let mut report = compute_performance(portfolio_id, &portfolio.currency, &transactions);

        // Unrealized P&L is a "now" snapshot and always uses the full
        // transaction history — restricting it to the report's date range
        // would give a nonsensical mid-period mark-to-market.
        let all_transactions = if from_date.is_some() || to_date.is_some() {
            self.transaction_repo
                .list_by_portfolio_chronological(portfolio_id)
                .await?
        } else {
            transactions
        };
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &all_transactions).await?;
        let mut positions = compute_positions(&all_transactions, &stocks);
        self.enrich_with_latest_prices(&mut positions).await?;

        let unrealized: Vec<f64> = positions.iter().filter_map(|p| p.unrealized_pnl).collect();
        report.unrealized_pnl_total = if unrealized.is_empty() {
            None
        } else {
            Some(unrealized.iter().sum())
        };

        Ok(report)
    }
}
