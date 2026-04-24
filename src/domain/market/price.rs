use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub id: i64,
    pub stock_id: i32,
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: String,
    pub fetched_at: NaiveDateTime,
}

/// A price point before it has been persisted (no id / fetched_at assigned yet).
#[derive(Debug, Clone)]
pub struct NewPrice {
    pub stock_id: i32,
    pub price_date: NaiveDate,
    pub close: Decimal,
    pub source: String,
}
