use garde::Validate;
use serde::Deserialize;

use crate::domain::market::stock::{NewStock, UpdateStock};

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateStockRequest {
    #[garde(length(min = 1, max = 8))]
    pub symbol: String,
    #[garde(length(min = 1, max = 120))]
    pub name: String,
    #[garde(pattern(r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$"))]
    pub isin: String,
    #[garde(length(max = 10))]
    pub currency: Option<String>,
    #[garde(length(max = 50))]
    pub market: Option<String>,
    #[garde(length(max = 50))]
    pub sector: Option<String>,
    #[garde(length(max = 50))]
    pub industry: Option<String>,
    #[garde(length(max = 8))]
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateStockRequest {
    #[garde(length(min = 1, max = 8))]
    pub symbol: String,
    #[garde(length(min = 1, max = 120))]
    pub name: String,
    #[garde(length(max = 10))]
    pub currency: Option<String>,
    #[garde(length(max = 50))]
    pub market: Option<String>,
    #[garde(length(max = 50))]
    pub sector: Option<String>,
    #[garde(length(max = 50))]
    pub industry: Option<String>,
    #[garde(length(max = 8))]
    pub country: Option<String>,
}

impl From<UpdateStockRequest> for UpdateStock {
    fn from(req: UpdateStockRequest) -> Self {
        UpdateStock {
            symbol: req.symbol,
            name: req.name,
            currency: req.currency,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
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
