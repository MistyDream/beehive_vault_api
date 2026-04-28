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

    /// Returns `(items, truncated)`. `truncated` is `true` when the underlying
    /// repository capped the result set; the controller surfaces this to the
    /// client via a response header.
    pub async fn search(&self, query: String) -> Result<(Vec<Stock>, bool), AppError> {
        self.repo.search(query).await
    }
}
