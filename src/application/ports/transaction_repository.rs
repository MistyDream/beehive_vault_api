use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use crate::application::error::AppError;
use crate::domain::wallet::transaction::{NewTransaction, Transaction, TransactionFilter, UpdateTransaction};

pub trait TransactionRepository: Send + Sync {
    fn find_by_id(&self, portfolio_id: Uuid, tx_id: Uuid) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>>;

    fn list_by_portfolio(&self, portfolio_id: Uuid) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>>;

    fn list_by_portfolio_chronological(&self, portfolio_id: Uuid) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>>;

    fn list_by_portfolio_filtered(&self, portfolio_id: Uuid, filters: TransactionFilter) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>>;

    fn insert(&self, new: NewTransaction) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>>;

    fn update(&self, portfolio_id: Uuid, tx_id: Uuid, data: UpdateTransaction) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>>;

    fn delete(&self, portfolio_id: Uuid, tx_id: Uuid) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>>;
}
