use serde::{Deserialize, Serialize};

use crate::domain::market::enums::MarketRegion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: String,
    pub market_region: MarketRegion,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}
