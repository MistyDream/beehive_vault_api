use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}
