use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioScoring {
    pub portfolio_id: Uuid,
    pub stock_scores: Vec<StockScore>,
    /// Portfolio-weighted average score (None if no positions have scores).
    pub weighted_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScore {
    pub stock_id: i32,
    pub symbol: String,
    pub name: String,
    pub weight: f64,
    pub global_score: Option<f64>,
    pub scored_at: Option<NaiveDateTime>,
}
