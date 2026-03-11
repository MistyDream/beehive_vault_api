use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use crate::domain::market::enums::MetricPeriod;
use crate::domain::market::metric_value::MetricValue;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::metric_values;

#[derive(Queryable, Selectable)]
#[diesel(table_name = metric_values)]
pub struct MetricValueRow {
    pub id: i64,
    pub stock_id: i32,
    pub metric_key: String,
    pub period: String,
    pub period_end: NaiveDate,
    pub value: f64,
    pub unit: Option<String>,
    pub currency: Option<String>,
    pub source: String,
    pub fetched_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = metric_values)]
pub struct NewMetricValueRow<'a> {
    pub stock_id: i32,
    pub metric_key: &'a str,
    pub period: &'a str,
    pub period_end: NaiveDate,
    pub value: f64,
    pub unit: Option<&'a str>,
    pub currency: Option<&'a str>,
    pub source: &'a str,
    pub fetched_at: NaiveDateTime,
}

impl TryFrom<MetricValueRow> for MetricValue {
    type Error = DbError;

    fn try_from(row: MetricValueRow) -> Result<Self, Self::Error> {
        Ok(MetricValue {
            id: row.id,
            stock_id: row.stock_id,
            metric_key: row.metric_key,
            period: MetricPeriod::try_from(row.period.as_str())
                .map_err(DbError::Conversion)?,
            period_end: row.period_end,
            value: row.value,
            unit: row.unit,
            currency: row.currency,
            source: row.source,
            fetched_at: row.fetched_at,
        })
    }
}
