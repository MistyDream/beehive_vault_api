use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::application::services::pagination::{paginate_slice, Paginated};
use crate::domain::market::stock::Stock;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::{NewTransaction, Transaction, TransactionFilter, UpdateTransaction};

#[derive(Debug, Default, Clone)]
pub struct TransactionsQuery {
    pub filters: TransactionFilter,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
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

    async fn fetch_stocks_for(&self, transactions: &[Transaction]) -> Result<HashMap<i32, Stock>, AppError> {
        let stock_ids: Vec<i32> = transactions
            .iter()
            .filter_map(|t| t.stock_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let stocks = self.stock_repo.find_by_ids(stock_ids).await?;
        Ok(stocks.into_iter().map(|s| (s.id, s)).collect())
    }

    async fn fetch_stock_for(&self, transaction: &Transaction) -> Result<HashMap<i32, Stock>, AppError> {
        match transaction.stock_id {
            Some(id) => {
                let stocks = self.stock_repo.find_by_ids(vec![id]).await?;
                Ok(stocks.into_iter().map(|s| (s.id, s)).collect())
            }
            None => Ok(HashMap::new()),
        }
    }

    pub async fn create(
        &self,
        portfolio_id: i32,
        new: NewTransaction,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        self.portfolio_repo.find_by_id(portfolio_id).await?;
        Self::validate(&new)?;
        let transaction = self.transaction_repo.insert(new).await?;
        let stocks = self.fetch_stock_for(&transaction).await?;
        Ok((transaction, stocks))
    }

    pub async fn get(
        &self,
        portfolio_id: i32,
        tx_id: i64,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        let transaction = self.transaction_repo.find_by_id(portfolio_id, tx_id).await?;
        let stocks = self.fetch_stock_for(&transaction).await?;
        Ok((transaction, stocks))
    }

    pub async fn list_paginated(
        &self,
        portfolio_id: i32,
        query: TransactionsQuery,
    ) -> Result<(Paginated<Transaction>, HashMap<i32, Stock>), AppError> {
        let has_filters = query.filters.transaction_type.is_some()
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

        let sort_by = query.sort_by.as_deref().unwrap_or("executed_at");
        let ascending = matches!(query.sort_dir.as_deref(), Some("asc"));
        transactions.sort_by(|a, b| {
            let ord = match sort_by {
                "amount" => a
                    .amount
                    .unwrap_or(0.0)
                    .partial_cmp(&b.amount.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal),
                "transaction_type" => a
                    .transaction_type
                    .as_str()
                    .cmp(b.transaction_type.as_str()),
                _ => a.executed_at.cmp(&b.executed_at),
            };
            if ascending { ord } else { ord.reverse() }
        });

        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(25);
        let paginated = paginate_slice(transactions, page, limit);
        let stocks = self.fetch_stocks_for(&paginated.items).await?;
        Ok((paginated, stocks))
    }

    pub async fn update(
        &self,
        portfolio_id: i32,
        tx_id: i64,
        data: UpdateTransaction,
    ) -> Result<(Transaction, HashMap<i32, Stock>), AppError> {
        Self::validate(&data)?;
        let transaction = self.transaction_repo.update(portfolio_id, tx_id, data).await?;
        let stocks = self.fetch_stock_for(&transaction).await?;
        Ok((transaction, stocks))
    }

    pub async fn delete(&self, portfolio_id: i32, tx_id: i64) -> Result<(), AppError> {
        let deleted = self.transaction_repo.delete(portfolio_id, tx_id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    fn validate(tx: &NewTransaction) -> Result<(), AppError> {
        match tx.transaction_type {
            TransactionType::Buy | TransactionType::Sell => {
                if tx.stock_id.is_none() {
                    return Err(AppError::BadRequest(format!("{} requires stock_id", tx.transaction_type.as_str())));
                }
                if tx.quantity.is_none() {
                    return Err(AppError::BadRequest(format!("{} requires quantity", tx.transaction_type.as_str())));
                }
                if tx.unit_price.is_none() {
                    return Err(AppError::BadRequest(format!("{} requires unit_price", tx.transaction_type.as_str())));
                }
            }
            TransactionType::Dividend => {
                if tx.stock_id.is_none() {
                    return Err(AppError::BadRequest("dividend requires stock_id".to_owned()));
                }
                if tx.amount.is_none() {
                    return Err(AppError::BadRequest("dividend requires amount".to_owned()));
                }
            }
            TransactionType::Fee => {
                if tx.amount.is_none() {
                    return Err(AppError::BadRequest("fee requires amount".to_owned()));
                }
            }
            TransactionType::Split => {
                if tx.stock_id.is_none() {
                    return Err(AppError::BadRequest("split requires stock_id".to_owned()));
                }
                if tx.split_from.is_none() || tx.split_to.is_none() {
                    return Err(AppError::BadRequest("split requires split_from and split_to".to_owned()));
                }
            }
            TransactionType::Deposit | TransactionType::Withdrawal => {
                if tx.amount.is_none() {
                    return Err(AppError::BadRequest(format!("{} requires amount", tx.transaction_type.as_str())));
                }
            }
        }
        Ok(())
    }
}
