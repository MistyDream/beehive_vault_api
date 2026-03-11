use diesel::prelude::*;

use crate::domain::market::stock::Stock;
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
pub struct NewStockRow<'a> {
    pub symbol: &'a str,
    pub name: &'a str,
    pub isin: &'a str,
    pub currency: Option<&'a str>,
    pub market: Option<&'a str>,
    pub sector: Option<&'a str>,
    pub industry: Option<&'a str>,
    pub country: Option<&'a str>,
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
