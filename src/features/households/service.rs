use crate::{
    error::{ApiError, required_text},
    types::{CurrencyCode, HouseholdId},
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
            timezone: required_text(command.timezone, "timezone")?,
        };
        Ok(self.repository.create(household).await?)
    }

    pub async fn get(&self, household_id: HouseholdId) -> Result<Household, ApiError> {
        self.repository
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))
    }
}
