use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::wallet::enums::PortfolioKind;
use crate::domain::wallet::portfolio::Portfolio;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::portfolios;

#[derive(Queryable, Selectable)]
#[diesel(table_name = portfolios)]
pub struct PortfolioRow {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// `id` is generated app-side (`Uuid::now_v7()`) at insert time so the row
/// carries a sortable timestamp prefix that helps B-tree locality. The DB has
/// a `gen_random_uuid()` default as a fallback for partial inserts.
#[derive(Insertable)]
#[diesel(table_name = portfolios)]
pub struct NewPortfolioRow<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub kind: &'a str,
    pub currency: &'a str,
    pub description: Option<&'a str>,
}

impl TryFrom<PortfolioRow> for Portfolio {
    type Error = DbError;

    fn try_from(row: PortfolioRow) -> Result<Self, Self::Error> {
        Ok(Portfolio {
            id: row.id,
            name: row.name,
            kind: PortfolioKind::try_from(row.kind.as_str())
                .map_err(DbError::Conversion)?,
            currency: row.currency,
            description: row.description,
            updated_at: row.updated_at,
        })
    }
}
