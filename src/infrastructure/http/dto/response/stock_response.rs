use serde::Serialize;

use crate::domain::market::enums::MarketRegion;
use crate::domain::market::isin::Isin;
use crate::domain::market::stock::Stock;

/// Slim DTO returned by `GET /v1/stocks?q=` — minimal payload tuned for the
/// frontend stock picker.
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

/// Full DTO returned by `GET /v1/stocks/{isin}`, `POST /v1/stocks`, and
/// `PATCH /v1/stocks/{isin}` — exposes every persisted field, in contrast to
/// the slim `StockResponse` used by the search endpoint.
#[derive(Serialize)]
pub struct StockDetailResponse {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: Isin,
    pub currency: String,
    pub market_region: MarketRegion,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

impl From<Stock> for StockDetailResponse {
    fn from(s: Stock) -> Self {
        StockDetailResponse {
            id: s.id,
            symbol: s.symbol,
            name: s.name,
            isin: s.isin,
            currency: s.currency,
            market_region: s.market_region,
            market: s.market,
            sector: s.sector,
            industry: s.industry,
            country: s.country,
        }
    }
}
