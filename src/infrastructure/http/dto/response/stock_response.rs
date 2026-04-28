use serde::Serialize;

use crate::domain::market::stock::Stock;

/// Slim summary projection of a stock, suitable for list/search results.
/// The full record (sector, industry, country, market_region, ...) is
/// reserved for a future detail endpoint.
#[derive(Serialize)]
pub struct StockResponse {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub currency: String,
}

impl From<Stock> for StockResponse {
    fn from(s: Stock) -> Self {
        StockResponse {
            id: s.id,
            symbol: s.symbol,
            name: s.name,
            currency: s.currency,
        }
    }
}
