use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// A sector/industry median value for a given metric, sourced from GuruFocus.
/// Used as the reference point for relative scoring (e.g. PER vs sector median).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorBenchmark {
    pub id: i32,
    pub sector: String,
    pub industry: Option<String>,
    pub metric_key: String,
    pub value: f64,
    pub source: String,
    pub period_end: NaiveDate,
    pub fetched_at: NaiveDateTime,
}
