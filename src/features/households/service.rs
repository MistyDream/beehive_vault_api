use crate::{
    error::{ApiError, ProblemKind, required_text},
    types::{CurrencyCode, HouseholdId, TimeZoneId},
};

use super::{
    domain::{Household, NewHousehold},
    repository::HouseholdRepository,
};

pub struct CreateHouseholdCommand {
    pub name: String,
    pub base_currency: CurrencyCode,
    pub timezone: String,
}

#[derive(Clone)]
pub struct HouseholdService {
    repository: HouseholdRepository,
}

impl HouseholdService {
    pub fn new(repository: HouseholdRepository) -> Self {
        Self { repository }
    }

    pub async fn create(&self, command: CreateHouseholdCommand) -> Result<Household, ApiError> {
        let household = NewHousehold {
            id: HouseholdId::new(),
            name: required_text(command.name, "name")?,
            base_currency: command.base_currency,
            timezone: TimeZoneId::new(command.timezone).map_err(|error| {
                ApiError::body_validation("#/timezone", "invalid_timezone", error.to_string())
            })?,
        };
        Ok(self.repository.create(household).await?)
    }

    pub async fn list(&self) -> Result<Vec<Household>, ApiError> {
        Ok(self.repository.list().await?)
    }

    pub async fn get(&self, household_id: HouseholdId) -> Result<Household, ApiError> {
        self.repository
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))
    }
}
