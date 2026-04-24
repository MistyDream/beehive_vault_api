use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use chrono::NaiveDate;

use crate::application::error::AppError;
use crate::domain::market::price::{NewPrice, Price};

pub trait StockPriceRepository: Send + Sync {
    /// Upsert prices on `(stock_id, price_date)`. Returns the number of rows
    /// affected (inserted + updated). Callers pre-compute the batch.
    fn upsert_many(
        &self,
        prices: Vec<NewPrice>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>>;

    /// Return the most recent persisted price for `stock_id`, if any.
    fn find_latest(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Price>, AppError>> + Send + '_>>;

    /// Latest price per stock for a batch of stock ids (used to valorize a
    /// whole portfolio in a single DB round-trip). Missing entries are omitted.
    fn find_latest_batch(
        &self,
        stock_ids: Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<i32, Price>, AppError>> + Send + '_>>;

    /// Return prices for `stock_id` between `from` and `to` (inclusive), sorted ascending.
    fn find_history(
        &self,
        stock_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Price>, AppError>> + Send + '_>>;
}
