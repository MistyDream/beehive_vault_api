use serde::{Deserialize, Serialize};

use crate::domain::market::enums::MetricCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDetail {
    pub id: i32,
    pub snapshot_id: i32,
    pub category: MetricCategory,
    pub score: f64,
    pub weight: f64,
}
