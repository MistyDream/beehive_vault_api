use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::domain::scoring::enums::MetricPeriod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub id: i64,
    pub stock_id: i32,
    pub metric_key: String,
    pub period: MetricPeriod,
    pub period_end: NaiveDate,
    pub value: f64,
    pub unit: Option<String>,
    pub currency: Option<String>,
    pub source: String,
    pub fetched_at: NaiveDateTime,
}
