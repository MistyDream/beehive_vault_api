use chrono::{DateTime, Utc};

use crate::types::{CurrencyCode, HouseholdId, TimeZoneId};

pub struct Household {
    pub id: HouseholdId,
    pub name: String,
    pub base_currency: CurrencyCode,
    pub timezone: TimeZoneId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewHousehold {
    pub id: HouseholdId,
    pub name: String,
    pub base_currency: CurrencyCode,
    pub timezone: TimeZoneId,
}
