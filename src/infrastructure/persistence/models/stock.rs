use diesel::prelude::*;

use crate::domain::market::stock::{NewStock, Stock, UpdateStock};
use crate::schema::stocks;

#[derive(Queryable, Selectable)]
#[diesel(table_name = stocks)]
pub struct StockRow {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = stocks)]
pub struct NewStockRow {
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = stocks)]
pub struct UpdateStockRow {
    pub symbol: String,
    pub name: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl From<UpdateStock> for UpdateStockRow {
    fn from(s: UpdateStock) -> Self {
        UpdateStockRow {
            symbol: s.symbol,
            name: s.name,
            currency: s.currency,
            market: s.market,
            sector: s.sector,
            industry: s.industry,
            country: s.country,
            updated_at: Some(chrono::Utc::now().naive_utc()),
        }
    }
}

impl From<NewStock> for NewStockRow {
    fn from(s: NewStock) -> Self {
        NewStockRow {
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

impl From<StockRow> for Stock {
    fn from(row: StockRow) -> Self {
        Stock {
            id: row.id,
            symbol: row.symbol,
            name: row.name,
            isin: row.isin,
            currency: row.currency,
            market: row.market,
            sector: row.sector,
            industry: row.industry,
            country: row.country,
        }
    }
}
