use serde::Serialize;

use crate::domain::market::stock::Stock;

#[derive(Serialize)]
pub struct StockResponse {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: String,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

impl From<Stock> for StockResponse {
    fn from(s: Stock) -> Self {
        StockResponse {
            id: s.id,
            symbol: s.symbol,
            name: s.name,
            isin: s.isin,
            currency: s.currency,
            market: s.market,
            sector: s.sector,
            industry: s.industry,
            country: s.country,
        }
    }
}
