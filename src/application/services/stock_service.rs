use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::Stock;

pub struct StockService {
    repo: Arc<dyn StockRepository>,
}

impl StockService {
    pub fn new(repo: Arc<dyn StockRepository>) -> Self {
        Self { repo }
    }

    pub async fn search(&self, query: String) -> Result<Vec<Stock>, AppError> {
        self.repo.search(query).await
    }
}
