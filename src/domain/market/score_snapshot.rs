use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSnapshot {
    pub id: i32,
    pub stock_id: i32,
    pub scored_at: NaiveDateTime,
    pub global_score: f64,
}
