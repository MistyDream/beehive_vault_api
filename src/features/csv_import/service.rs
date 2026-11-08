use std::collections::{HashMap, HashSet};

use chrono::{NaiveDate, Utc};
use sha2::{Digest, Sha256};

use crate::{
    error::{ApiError, ProblemKind},
    features::{accounts::AccountRepository, households::HouseholdRepository},
    types::{AccountId, CurrencyCode, HouseholdId},
};

use super::{
    domain::{
        CsvFormat, ImportedRow, MAX_CSV_BYTES, MAX_CSV_ROWS, PreviewRow, RowSelection, SelectedRow,
        preview_rows,
    },
    repository::CsvImportRepository,
};

#[derive(Clone)]
pub(super) struct CsvImportService {
    accounts: AccountRepository,
    households: HouseholdRepository,
    imports: CsvImportRepository,
}

impl CsvImportService {
    pub fn new(
        accounts: AccountRepository,
        households: HouseholdRepository,
        imports: CsvImportRepository,
    ) -> Self {
        Self {
            accounts,
            households,
            imports,
        }
    }

    pub async fn preview(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        csv: &str,
        format: &CsvFormat,
    ) -> Result<(CurrencyCode, Vec<PreviewRow>), ApiError> {
        validate_csv(csv, format)?;
        let today = self.household_today(household_id).await?;
        let account = self
            .accounts
            .find(household_id, account_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))?;
        let rows = preview_rows(csv, format, today)?;

        Ok((account.currency, rows))
    }

    pub async fn confirm(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        csv: &str,
        format: &CsvFormat,
        file_name: &str,
        selections: Vec<RowSelection>,
    ) -> Result<Vec<ImportedRow>, ApiError> {
        validate_csv(csv, format)?;
        let file_name = file_name.trim();
        if !(1..=255).contains(&file_name.chars().count()) || file_name.contains(char::is_control) {
            return Err(ApiError::body_validation(
                "#/fileName",
                "invalid_file_name",
                "The file name must contain between 1 and 255 characters and no control characters.",
            ));
        }
        if !(1..=MAX_CSV_ROWS).contains(&selections.len()) {
            return Err(ApiError::body_validation(
                "#/rows",
                "invalid_selection_size",
                "Select between 1 and 5000 data records.",
            ));
        }
        let today = self.household_today(household_id).await?;
        let mut rows_by_line: HashMap<_, _> = preview_rows(csv, format, today)?
            .into_iter()
            .map(|row| (row.line, row))
            .collect();
        let mut selected_lines = HashSet::new();
        let mut selected = Vec::with_capacity(selections.len());
        for (index, selection) in selections.into_iter().enumerate() {
            let problem = |code, detail| {
                ApiError::body_validation(format!("#/rows/{index}/line"), code, detail)
            };
            if !selected_lines.insert(selection.line) {
                return Err(problem(
                    "duplicate_csv_selection",
                    "Select a CSV line only once.",
                ));
            }
            let row = usize::try_from(selection.line)
                .ok()
                .and_then(|line| rows_by_line.remove(&line))
                .ok_or_else(|| {
                    problem(
                        "csv_line_not_found",
                        "The selected CSV line does not start a data record.",
                    )
                })?;
            let values = row.result.map_err(|_| {
                problem(
                    "rejected_csv_line",
                    "The selected CSV line was rejected in preview.",
                )
            })?;
            selected.push(SelectedRow {
                selection,
                values,
                source_record: &csv[row.source_range],
            });
        }

        let statement_digest = Sha256::digest(csv.as_bytes());
        self.imports
            .confirm(
                household_id,
                account_id,
                &statement_digest,
                file_name,
                selected,
            )
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))
    }

    async fn household_today(&self, household_id: HouseholdId) -> Result<NaiveDate, ApiError> {
        let household = self
            .households
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        household
            .timezone
            .date_at(Utc::now())
            .map_err(|error| ApiError::from(sqlx::Error::Decode(Box::new(error))))
    }
}

fn validate_csv(csv: &str, format: &CsvFormat) -> Result<(), ApiError> {
    if csv.len() > MAX_CSV_BYTES {
        return Err(ApiError::new(ProblemKind::PayloadTooLarge)
            .with_detail("The CSV text exceeds the supported size of 1 MiB."));
    }
    format.validate()?;
    Ok(())
}
