use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;
use crate::domain::market::stock::Stock;

pub trait StockRepository: Send + Sync {
    fn find_by_id(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn find_by_symbol(&self, symbol: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn find_by_isin(&self, isin: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn list_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>>;

    fn delete(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>>;
}
