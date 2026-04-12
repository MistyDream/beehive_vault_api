use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::wallet::enums::PortfolioKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: i32,
    pub name: String,
    pub kind: PortfolioKind,
    pub currency: String,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}
