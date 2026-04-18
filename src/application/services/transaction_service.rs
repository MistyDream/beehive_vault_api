use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::{NewTransaction, Transaction, TransactionFilter, UpdateTransaction};

pub struct TransactionService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl TransactionService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo }
    }

    pub async fn create(&self, portfolio_id: i32, new: NewTransaction) -> Result<Transaction, AppError> {
        // Verify portfolio exists
        self.portfolio_repo.find_by_id(portfolio_id).await?;
        // Validate business rules
        Self::validate(&new)?;
        self.transaction_repo.insert(new).await
    }

    pub async fn get(&self, portfolio_id: i32, tx_id: i64) -> Result<Transaction, AppError> {
        self.transaction_repo.find_by_id(portfolio_id, tx_id).await
    }

    pub async fn list(&self, portfolio_id: i32, filters: Option<TransactionFilter>) -> Result<Vec<Transaction>, AppError> {
        match filters {
            Some(f) => self.transaction_repo.list_by_portfolio_filtered(portfolio_id, f).await,
            None => self.transaction_repo.list_by_portfolio(portfolio_id).await,
        }
    }

    pub async fn update(&self, portfolio_id: i32, tx_id: i64, data: UpdateTransaction) -> Result<Transaction, AppError> {
        Self::validate(&data)?;
        self.transaction_repo.update(portfolio_id, tx_id, data).await
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
