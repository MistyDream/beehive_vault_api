use garde::Validate;
use serde::Deserialize;

use crate::domain::market::enums::MarketRegion;
use crate::domain::market::stock::{NewStock, StockPatch};

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct StockSearchQuery {
    /// Optional: a missing `q` produces a 400 from the controller; a present
    /// but invalid `q` produces a 422 from this validator.
    #[garde(inner(length(chars, max = 50), custom(non_blank_min_2)))]
    pub q: Option<String>,
}

fn non_blank_min_2(value: &String, _: &()) -> garde::Result {
    if value.trim().chars().count() < 2 {
        return Err(garde::Error::new(
            "must contain at least 2 non-whitespace characters",
        ));
    }
    Ok(())
}

// Patterns inlined into `garde(pattern(...))` directly — the macro needs a
// literal: ISIN (ISO 6166: 2-letter country, 9 alphanumerics, 1 check digit
// — format only, check digit not verified), currency (ISO 4217), country (ISO
// 3166-1 alpha-2).
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateStockRequest {
    #[garde(length(chars, min = 1, max = 20))]
    pub symbol: String,
    #[garde(length(chars, min = 1, max = 255))]
    pub name: String,
    #[garde(pattern(r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$"))]
    pub isin: String,
    #[garde(pattern(r"^[A-Z]{3}$"))]
    pub currency: String,
    #[garde(skip)]
    pub market_region: MarketRegion,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub market: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub sector: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub industry: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}$")))]
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateStockRequest {
    #[garde(inner(length(chars, min = 1, max = 20)))]
    pub symbol: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 255)))]
    pub name: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$")))]
    pub isin: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{3}$")))]
    pub currency: Option<String>,
    #[garde(skip)]
    pub market_region: Option<MarketRegion>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub market: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub sector: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub industry: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}$")))]
    pub country: Option<String>,
}

impl From<CreateStockRequest> for NewStock {
    fn from(req: CreateStockRequest) -> Self {
        NewStock {
            symbol: req.symbol,
            name: req.name,
            isin: req.isin,
            currency: req.currency,
            market_region: req.market_region,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
}

impl From<UpdateStockRequest> for StockPatch {
    fn from(req: UpdateStockRequest) -> Self {
        StockPatch {
            symbol: req.symbol,
            name: req.name,
            isin: req.isin,
            currency: req.currency,
            market_region: req.market_region,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
}
