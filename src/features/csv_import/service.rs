use chrono::Utc;

use crate::{
    error::{ApiError, ProblemKind},
    features::{accounts::AccountRepository, households::HouseholdRepository},
    types::{AccountId, CurrencyCode, HouseholdId},
};

use super::domain::{CsvFormat, MAX_CSV_BYTES, PreviewRow, preview_rows};

#[derive(Clone)]
pub(super) struct CsvImportService {
    accounts: AccountRepository,
    households: HouseholdRepository,
}

impl CsvImportService {
    pub fn new(accounts: AccountRepository, households: HouseholdRepository) -> Self {
        Self {
            accounts,
            households,
        }
    }

    pub async fn preview(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        csv: &str,
        format: &CsvFormat,
    ) -> Result<(CurrencyCode, Vec<PreviewRow>), ApiError> {
        if csv.len() > MAX_CSV_BYTES {
            return Err(ApiError::new(ProblemKind::PayloadTooLarge)
                .with_detail("The CSV text exceeds the supported size of 1 MiB."));
        }
        format.validate()?;

        let household = self
            .households
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        let account = self
            .accounts
            .find(household_id, account_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))?;
        let today = household
            .timezone
            .date_at(Utc::now())
            .map_err(|error| ApiError::from(sqlx::Error::Decode(Box::new(error))))?;
        let rows = preview_rows(csv, format, today)?;

        Ok((account.currency, rows))
    }
}
