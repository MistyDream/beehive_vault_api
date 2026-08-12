use chrono::{DateTime, Utc};

use crate::types::{CurrencyCode, HouseholdId};

pub struct Household {
    pub id: HouseholdId,
    pub name: String,
    pub base_currency: CurrencyCode,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewHousehold {
    pub id: HouseholdId,
    pub name: String,
    pub base_currency: CurrencyCode,
    pub timezone: String,
}
