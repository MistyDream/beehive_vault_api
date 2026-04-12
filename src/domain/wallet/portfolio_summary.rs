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
}
