use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use crate::domain::market::sector_benchmark::SectorBenchmark;
use crate::schema::sector_benchmarks;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sector_benchmarks)]
pub struct SectorBenchmarkRow {
    pub id: i32,
    pub sector: String,
    pub industry: Option<String>,
    pub metric_key: String,
    pub value: f64,
    pub source: String,
    pub period_end: NaiveDate,
    pub fetched_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = sector_benchmarks)]
pub struct NewSectorBenchmarkRow<'a> {
    pub sector: &'a str,
    pub industry: Option<&'a str>,
    pub metric_key: &'a str,
    pub value: f64,
    pub source: &'a str,
    pub period_end: NaiveDate,
    pub fetched_at: NaiveDateTime,
}

impl From<SectorBenchmarkRow> for SectorBenchmark {
    fn from(row: SectorBenchmarkRow) -> Self {
        SectorBenchmark {
            id: row.id,
            sector: row.sector,
            industry: row.industry,
            metric_key: row.metric_key,
            value: row.value,
            source: row.source,
            period_end: row.period_end,
            fetched_at: row.fetched_at,
        }
    }
}
