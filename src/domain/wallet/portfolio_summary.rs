use serde::{Deserialize, Serialize};

use crate::domain::wallet::cash_balance::CashBalance;
use crate::domain::wallet::portfolio::Portfolio;
use crate::domain::wallet::position::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub portfolio: Portfolio,
    pub positions: Vec<Position>,
    pub cash: CashBalance,
    pub total_invested: f64,
    /// `cash.balance + Σ positions.current_value` where known. Partial when
    /// some positions lack a price — consult `positions_without_price`.
    pub total_value: f64,
    /// Number of positions whose `current_value` is `None` (missing price).
    /// Zero means `total_value` is complete.
    pub positions_without_price: usize,
}
