use serde::Deserialize;

use crate::domain::market::stock::StockFilter;

#[derive(Debug, Deserialize)]
pub struct ListStocksQuery {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub isin: Option<String>,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl From<ListStocksQuery> for StockFilter {
    fn from(q: ListStocksQuery) -> Self {
        StockFilter {
            symbol: q.symbol,
            name: q.name,
            isin: q.isin,
            currency: q.currency,
            market: q.market,
            sector: q.sector,
            industry: q.industry,
            country: q.country,
            page: q.page.unwrap_or(1).max(1),
            per_page: q.per_page.unwrap_or(20).clamp(1, 100),
        }
    }
}
