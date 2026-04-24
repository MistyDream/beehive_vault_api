use std::sync::Arc;

use chrono::{Days, NaiveDate};

use crate::application::error::AppError;
use crate::application::ports::stock_price_repository::StockPriceRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::price::Price;
use crate::domain::market::stock::Stock;

/// Defence-in-depth cap on the price history window. The storage itself is
/// already bounded by the 5-year backfill policy, but this guard survives any
/// future extension of that policy and keeps a single unbounded client query
/// from scanning the whole table.
const MAX_HISTORY_WINDOW_DAYS: u64 = 365 * 10;

pub struct PriceService {
    stock_repo: Arc<dyn StockRepository>,
    price_repo: Arc<dyn StockPriceRepository>,
}

impl PriceService {
    pub fn new(
        stock_repo: Arc<dyn StockRepository>,
        price_repo: Arc<dyn StockPriceRepository>,
    ) -> Self {
        Self { stock_repo, price_repo }
    }

    /// Return the most recent close for `stock_id` along with the stock's currency.
    /// Errors: `NotFound` if the stock does not exist, `NotFound` if no price exists.
    pub async fn get_latest(&self, stock_id: i32) -> Result<(Price, Stock), AppError> {
        let stock = self.stock_repo.find_by_id(stock_id).await?;
        let price = self
            .price_repo
            .find_latest(stock_id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok((price, stock))
    }

    /// Return all persisted closes for `stock_id` between `from` and `to` (inclusive)
    /// along with the stock (used by callers to expose the currency).
    /// Errors: `NotFound` if the stock does not exist,
    /// `BadRequest` if `from > to`.
    pub async fn get_history(
        &self,
        stock_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<(Vec<Price>, Stock), AppError> {
        if from > to {
            return Err(AppError::BadRequest(
                "query parameter 'from' must be on or before 'to'".to_string(),
            ));
        }
        if from
            .checked_add_days(Days::new(MAX_HISTORY_WINDOW_DAYS))
            .is_none_or(|cap| to > cap)
        {
            return Err(AppError::BadRequest(format!(
                "price history window must not exceed {} days",
                MAX_HISTORY_WINDOW_DAYS
            )));
        }
        let stock = self.stock_repo.find_by_id(stock_id).await?;
        let prices = self.price_repo.find_history(stock_id, from, to).await?;
        Ok((prices, stock))
    }
}
