use diesel::prelude::*;

use crate::domain::market::enums::MarketRegion;
use crate::domain::market::isin::Isin;
use crate::domain::market::stock::Stock;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::stocks;

#[derive(Queryable, Selectable)]
#[diesel(table_name = stocks)]
pub struct StockRow {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: String,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub market_region: String,
}

#[derive(Insertable)]
#[diesel(table_name = stocks)]
pub struct NewStockRow<'a> {
    pub symbol: &'a str,
    pub name: &'a str,
    pub isin: &'a str,
    pub currency: &'a str,
    pub market_region: &'a str,
    pub market: Option<&'a str>,
    pub sector: Option<&'a str>,
    pub industry: Option<&'a str>,
    pub country: Option<&'a str>,
}

impl TryFrom<StockRow> for Stock {
    type Error = DbError;

    fn try_from(row: StockRow) -> Result<Self, Self::Error> {
        Ok(Stock {
            id: row.id,
            symbol: row.symbol,
            name: row.name,
            isin: Isin::try_new(&row.isin)
                .map_err(|e| DbError::Conversion(format!("invalid isin in DB: {e}")))?,
            currency: row.currency,
            market_region: MarketRegion::try_from(row.market_region.as_str())
                .map_err(DbError::Conversion)?,
            market: row.market,
            sector: row.sector,
            industry: row.industry,
            country: row.country,
        })
    }
}
