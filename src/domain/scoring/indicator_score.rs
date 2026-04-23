use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorScore {
    pub id: i32,
    pub detail_id: i32,
    pub metric_key: String,
    pub score: f64,
}
