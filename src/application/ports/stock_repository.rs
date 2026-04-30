use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;
use crate::domain::market::enums::MarketRegion;
use crate::domain::market::stock::{NewStock, Stock, UpdateStock};

pub struct StockSearchResult {
    pub items: Vec<Stock>,
    pub truncated: bool,
}

pub trait StockRepository: Send + Sync {
    fn find_by_id(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn find_by_ids(&self, stock_ids: Vec<i32>) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>>;

    fn find_by_symbol(&self, symbol: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn find_by_isin(&self, isin: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn search(
        &self,
        query: String,
    ) -> Pin<Box<dyn Future<Output = Result<StockSearchResult, AppError>> + Send + '_>>;

    fn list_by_region(
        &self,
        region: MarketRegion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>>;

    fn insert(&self, new: NewStock) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn update(&self, stock_id: i32, data: UpdateStock) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>>;

    fn delete(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>>;
}
