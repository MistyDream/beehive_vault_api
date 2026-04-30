use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::{StockRepository, StockSearchResult};
use crate::domain::market::stock::{NewStock, Stock, StockPatch, UpdateStock};

pub struct StockService {
    repo: Arc<dyn StockRepository>,
}

impl StockService {
    pub fn new(repo: Arc<dyn StockRepository>) -> Self {
        Self { repo }
    }

    pub async fn search(&self, query: String) -> Result<StockSearchResult, AppError> {
        self.repo.search(query).await
    }

    pub async fn get_by_id(&self, stock_id: i32) -> Result<Stock, AppError> {
        self.repo.find_by_id(stock_id).await
    }

    pub async fn create(&self, new: NewStock) -> Result<Stock, AppError> {
        self.repo.insert(new).await
    }

    /// Apply a partial update by fetching the current stock, merging the patch,
    /// and writing the merged value back as a full replace at the repo layer.
    /// Last-write-wins on concurrent edits — acceptable for an admin CRUD.
    pub async fn update(&self, stock_id: i32, patch: StockPatch) -> Result<Stock, AppError> {
        let current = self.repo.find_by_id(stock_id).await?;
        let merged = UpdateStock {
            symbol: patch.symbol.unwrap_or(current.symbol),
            name: patch.name.unwrap_or(current.name),
            isin: patch.isin.unwrap_or(current.isin),
            currency: patch.currency.unwrap_or(current.currency),
            market_region: patch.market_region.unwrap_or(current.market_region),
            market: patch.market.or(current.market),
            sector: patch.sector.or(current.sector),
            industry: patch.industry.or(current.industry),
            country: patch.country.or(current.country),
        };
        self.repo.update(stock_id, merged).await
    }

    /// The DB enforces deletion safety: the `transactions.stock_id` FK is
    /// `ON DELETE RESTRICT`, so deleting a referenced stock surfaces as a 409
    /// (mapped from `ForeignKeyViolation` by the repo). Cascading FKs on
    /// prices/metrics/snapshots fire automatically — those are regeneratable.
    pub async fn delete(&self, stock_id: i32) -> Result<(), AppError> {
        let deleted = self.repo.delete(stock_id).await?;
        if !deleted {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
