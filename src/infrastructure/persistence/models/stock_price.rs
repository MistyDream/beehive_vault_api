use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rust_decimal::Decimal;

use crate::domain::market::price::Price;
use crate::schema::stock_prices;

#[derive(Queryable, Selectable)]
#[diesel(table_name = stock_prices)]
pub struct StockPriceRow {
    pub id: i64,
    pub stock_id: i32,
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: String,
    pub fetched_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = stock_prices)]
pub struct NewStockPriceRow<'a> {
    pub stock_id: i32,
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: &'a str,
}

impl From<StockPriceRow> for Price {
    fn from(row: StockPriceRow) -> Self {
        Price {
            id: row.id,
            stock_id: row.stock_id,
            price_date: row.price_date,
            close: row.close,
            source: row.source,
            fetched_at: row.fetched_at,
        }
    }
}
