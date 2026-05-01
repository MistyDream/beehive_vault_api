use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::application::services::pagination::{paginate_slice, Page, SortDirection, DEFAULT_LIMIT, DEFAULT_PAGE};
use crate::application::services::stock_lookup::{fetch_stock_by_id_optional, fetch_stocks_for_transactions};
use crate::domain::market::stock::Stock;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::{NewTransaction, Transaction, TransactionFilter, UpdateTransaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransactionSort {
    #[default]
    ExecutedAt,
    Amount,
    TransactionType,
}

impl TransactionSort {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("amount") => TransactionSort::Amount,
            Some("transaction_type") => TransactionSort::TransactionType,
            _ => TransactionSort::ExecutedAt,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TransactionsQuery {
    pub filters: TransactionFilter,
    pub sort_by: TransactionSort,
    pub sort_dir: SortDirection,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct TransactionStats {
    pub total: u64,
    pub buy: u64,
    pub sell: u64,
    pub dividend: u64,
    pub fee: u64,
    pub split: u64,
    pub deposit: u64,
    pub withdrawal: u64,
}

pub struct TransactionService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    stock_repo: Arc<dyn StockRepository>,
}

impl TransactionService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        stock_repo: Arc<dyn StockRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo, stock_repo }
    }

    pub async fn create(
        &self,
        portfolio_id: Uuid,
        new: NewTransaction,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        self.portfolio_repo.find_by_id(portfolio_id).await?;
        new.check_invariants().map_err(AppError::BadRequest)?;
        let transaction = self.transaction_repo.insert(new).await?;
        let stocks = fetch_stock_by_id_optional(&self.stock_repo, transaction.stock_id).await?;
        Ok((transaction, stocks))
    }

    pub async fn get(
        &self,
        portfolio_id: Uuid,
        tx_id: Uuid,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        let transaction = self.transaction_repo.find_by_id(portfolio_id, tx_id).await?;
        let stocks = fetch_stock_by_id_optional(&self.stock_repo, transaction.stock_id).await?;
        Ok((transaction, stocks))
    }

    pub async fn list_paginated(
        &self,
        portfolio_id: Uuid,
        query: TransactionsQuery,
    ) -> Result<(Page<Transaction>, HashMap<i32, Stock>), AppError> {
        let has_filters = !query.filters.transaction_types.is_empty()
            || query.filters.stock_id.is_some()
            || query.filters.from_date.is_some()
            || query.filters.to_date.is_some();

        let mut transactions = if has_filters {
            self.transaction_repo
                .list_by_portfolio_filtered(portfolio_id, query.filters.clone())
                .await?
        } else {
            self.transaction_repo.list_by_portfolio(portfolio_id).await?
        };

        let ascending = query.sort_dir == SortDirection::Asc;
        let tiebreaker = |a: &Transaction, b: &Transaction| a.id.cmp(&b.id);

        transactions.sort_by(|a, b| {
            let ord = match query.sort_by {
                TransactionSort::Amount => a
                    .amount
                    .unwrap_or(0.0)
                    .partial_cmp(&b.amount.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal),
                TransactionSort::TransactionType => a
                    .transaction_type
                    .as_str()
                    .cmp(b.transaction_type.as_str()),
                TransactionSort::ExecutedAt => a.executed_at.cmp(&b.executed_at),
            };
            let ord = if ascending { ord } else { ord.reverse() };
            ord.then_with(|| tiebreaker(a, b))
        });

        let page = query.page.unwrap_or(DEFAULT_PAGE);
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
        let paginated = paginate_slice(transactions, page, limit);
        let stocks = fetch_stocks_for_transactions(&self.stock_repo, &paginated.items).await?;
        Ok((paginated, stocks))
    }

    pub async fn stats(
        &self,
        portfolio_id: Uuid,
        stock_id: Option<i32>,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> Result<TransactionStats, AppError> {
        self.portfolio_repo.find_by_id(portfolio_id).await?;

        let has_filters = stock_id.is_some() || from_date.is_some() || to_date.is_some();
        let transactions = if has_filters {
            let filters = TransactionFilter {
                stock_id,
                from_date,
                to_date,
                ..Default::default()
            };
            self.transaction_repo
                .list_by_portfolio_filtered(portfolio_id, filters)
                .await?
        } else {
            self.transaction_repo.list_by_portfolio(portfolio_id).await?
        };

        let mut stats = TransactionStats::default();
        for tx in &transactions {
            match tx.transaction_type {
                TransactionType::Buy => stats.buy += 1,
                TransactionType::Sell => stats.sell += 1,
                TransactionType::Dividend => stats.dividend += 1,
                TransactionType::Fee => stats.fee += 1,
                TransactionType::Split => stats.split += 1,
                TransactionType::Deposit => stats.deposit += 1,
                TransactionType::Withdrawal => stats.withdrawal += 1,
            }
        }
        stats.total = transactions.len() as u64;
        Ok(stats)
    }

    pub async fn update(
        &self,
        portfolio_id: Uuid,
        tx_id: Uuid,
        data: UpdateTransaction,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        data.check_invariants().map_err(AppError::BadRequest)?;
        let transaction = self.transaction_repo.update(portfolio_id, tx_id, data).await?;
        let stocks = fetch_stock_by_id_optional(&self.stock_repo, transaction.stock_id).await?;
        Ok((transaction, stocks))
    }

    pub async fn delete(&self, portfolio_id: Uuid, tx_id: Uuid) -> Result<(), AppError> {
        let deleted = self.transaction_repo.delete(portfolio_id, tx_id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

}
