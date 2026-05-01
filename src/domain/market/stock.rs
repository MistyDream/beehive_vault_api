use serde::{Deserialize, Serialize};

use crate::domain::market::enums::MarketRegion;
use crate::domain::market::isin::Isin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
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

#[derive(Debug, Clone)]
pub struct NewStock {
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

pub type UpdateStock = NewStock;

/// Partial update payload: `None` means "leave this field untouched". The
/// service merges a `StockPatch` into the current `Stock` and forwards a full
/// `UpdateStock` to the repository — keeps the port flat (full replace) while
/// surfacing PATCH semantics at the API boundary.
#[derive(Debug, Default, Clone)]
pub struct StockPatch {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub isin: Option<Isin>,
    pub currency: Option<String>,
    pub market_region: Option<MarketRegion>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}
