use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::wallet::enums::PortfolioKind;
use crate::domain::wallet::portfolio::Portfolio;

#[derive(Serialize)]
pub struct PortfolioResponse {
    pub id: Uuid,
    pub name: String,
    pub kind: PortfolioKind,
    pub currency: String,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl From<Portfolio> for PortfolioResponse {
    fn from(p: Portfolio) -> Self {
        PortfolioResponse {
            id: p.id,
            name: p.name,
            kind: p.kind,
            currency: p.currency,
            description: p.description,
            updated_at: p.updated_at,
        }
    }
}
