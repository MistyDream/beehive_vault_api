use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::market::enums::{MetricCategory, MetricDataType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCatalog {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub category: MetricCategory,
    pub data_type: MetricDataType,
    pub unit: Option<String>,
    pub frequency: Option<String>,
    pub higher_is_better: bool,
    pub min_plausible: Option<f64>,
    pub max_plausible: Option<f64>,
    pub notes: Option<String>,
    pub updated_at: NaiveDateTime,
}
