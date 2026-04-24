//! Adapter implementing the `PriceFetcher` port against Yahoo Finance
//! via the `yfinance-rs` crate. Bound to the domain behind a trait so the
//! data source can be swapped (EODHD, Alpha Vantage, ...) without any
//! change in the application layer.

use std::future::Future;
use std::pin::Pin;

use chrono::{NaiveDate, TimeZone, Utc};
use yfinance_rs::{HistoryBuilder, YfClient};
use yfinance_rs::core::YfError;

use crate::application::error::AppError;
use crate::application::ports::price_fetcher::{FetchedPrice, PriceFetcher};

/// Identifier stored in `stock_prices.source` for rows written by this adapter.
pub const SOURCE: &str = "yahoo";

#[derive(Clone)]
pub struct YFinancePriceFetcher {
    client: YfClient,
}

impl YFinancePriceFetcher {
    pub fn new() -> Self {
        Self {
            client: YfClient::default(),
        }
    }
}

impl Default for YFinancePriceFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceFetcher for YFinancePriceFetcher {
    fn fetch_history(
        &self,
        symbol: String,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FetchedPrice>, AppError>> + Send + '_>> {
        Box::pin(async move {
            let start = Utc
                .from_utc_datetime(&from.and_hms_opt(0, 0, 0).expect("valid midnight"));
            let end = Utc
                .from_utc_datetime(&to.and_hms_opt(23, 59, 59).expect("valid end-of-day"));

            let candles = HistoryBuilder::new(&self.client, symbol)
                .between(start, end)
                .fetch()
                .await
                .map_err(yf_error_to_app)?;

            Ok(candles
                .into_iter()
                .map(|c| FetchedPrice {
                    price_date: c.ts.date_naive(),
                    close: c.close.amount(),
                    source: SOURCE.to_string(),
                })
                .collect())
        })
    }
}

fn yf_error_to_app(err: YfError) -> AppError {
    AppError::Internal(Box::new(err))
}
