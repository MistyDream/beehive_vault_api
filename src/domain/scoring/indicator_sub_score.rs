use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSubScore {
    pub id: i32,
    pub indicator_score_id: i32,
    pub sub_score_type: String,
    pub score: f64,
}
