use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::{NewStock, Paginated, Stock, StockFilter, UpdateStock};

pub struct StockService {
    repo: Arc<dyn StockRepository>,
}

impl StockService {
    pub fn new(repo: Arc<dyn StockRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_stock(&self, new: NewStock) -> Result<Stock, AppError> {
        self.repo.insert(new).await
    }

    pub async fn get_stock_by_isin(&self, isin: String) -> Result<Stock, AppError> {
        self.repo.find_by_isin(isin).await
    }

    pub async fn search_stocks(&self, filter: StockFilter) -> Result<Paginated<Stock>, AppError> {
        self.repo.search(filter).await
    }

    pub async fn update_stock(&self, isin: String, data: UpdateStock) -> Result<Stock, AppError> {
        self.repo.update(isin, data).await
    }

    pub async fn delete_stock(&self, isin: String) -> Result<(), AppError> {
        let deleted = self.repo.delete_by_isin(isin).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}
