use serde::Deserialize;

use crate::domain::market::stock::NewStock;

#[derive(Debug, Deserialize)]
pub struct CreateStockRequest {
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

impl From<CreateStockRequest> for NewStock {
    fn from(req: CreateStockRequest) -> Self {
        NewStock {
            symbol: req.symbol,
            name: req.name,
            isin: req.isin,
            currency: req.currency,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
}
