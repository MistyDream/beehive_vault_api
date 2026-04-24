use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::domain::market::price::Price;

/// A single price point exposed to API clients.
/// `close` is serialized as a string to preserve decimal precision (crate
/// feature `rust_decimal/serde-str`).
#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub currency: String,
    pub source: String,
}

impl PriceResponse {
    pub fn from_price(price: Price, currency: String) -> Self {
        Self {
            price_date: price.price_date,
            close: price.close,
            currency,
            source: price.source,
        }
    }
}

/// Envelope returned by the price history endpoint. Currency is lifted out of
/// each point since it is invariant for a given stock.
#[derive(Debug, Serialize)]
pub struct PriceHistoryResponse {
    pub currency: String,
    pub prices: Vec<PricePoint>,
}

#[derive(Debug, Serialize)]
pub struct PricePoint {
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: String,
}

impl PriceHistoryResponse {
    pub fn from_prices(prices: Vec<Price>, currency: String) -> Self {
        Self {
            currency,
            prices: prices
                .into_iter()
                .map(|p| PricePoint {
                    price_date: p.price_date,
                    close: p.close,
                    source: p.source,
                })
                .collect(),
        }
    }
}
