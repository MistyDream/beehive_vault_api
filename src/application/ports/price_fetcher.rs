use std::future::Future;
use std::pin::Pin;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::application::error::AppError;

/// A closing price as returned by an external market-data provider,
/// before any mapping to an internal `stock_id`. `source` identifies the
/// provider and is persisted as-is, so callers in the application layer do
/// not depend on any infrastructure-level adapter constant.
#[derive(Debug, Clone)]
pub struct FetchedPrice {
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: String,
}

pub trait PriceFetcher: Send + Sync {
    /// Return the closing prices for `symbol` between `from` and `to` (inclusive),
    /// sorted by ascending `price_date`. Weekends / holidays are naturally absent.
    fn fetch_history(
        &self,
        symbol: String,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FetchedPrice>, AppError>> + Send + '_>>;
}
