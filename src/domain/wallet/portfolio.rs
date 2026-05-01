use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::wallet::enums::PortfolioKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: Uuid,
    pub name: String,
    pub kind: PortfolioKind,
    pub currency: String,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

pub struct NewPortfolio {
    pub name: String,
    pub kind: PortfolioKind,
    pub currency: String,
    pub description: Option<String>,
}

pub type UpdatePortfolio = NewPortfolio;
