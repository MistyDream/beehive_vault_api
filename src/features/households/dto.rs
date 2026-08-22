use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{CurrencyCode, HouseholdId, TimeZoneId};

use super::{domain::Household, service::CreateHouseholdCommand};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHouseholdRequest {
    name: String,
    base_currency: CurrencyCode,
    timezone: String,
}

impl From<CreateHouseholdRequest> for CreateHouseholdCommand {
    fn from(request: CreateHouseholdRequest) -> Self {
        Self {
            name: request.name,
            base_currency: request.base_currency,
            timezone: request.timezone,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdResponse {
    id: HouseholdId,
    name: String,
    base_currency: CurrencyCode,
    timezone: TimeZoneId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Household> for HouseholdResponse {
    fn from(household: Household) -> Self {
        Self {
            id: household.id,
            name: household.name,
            base_currency: household.base_currency,
            timezone: household.timezone,
            created_at: household.created_at,
            updated_at: household.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn unknown_create_fields_are_ignored() {
        let request: CreateHouseholdRequest = serde_json::from_value(json!({
            "name": "Home",
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris",
            "futureField": "ignored"
        }))
        .expect("unknown fields should preserve forward compatibility");
        let command = CreateHouseholdCommand::from(request);

        assert_eq!(command.name, "Home");
    }
}
