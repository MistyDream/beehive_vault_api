use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::{NewStock, Stock};

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
}
