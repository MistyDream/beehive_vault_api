use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    error::ApiError,
    features::{
        accounts::AccountRepository, categories::CategoryRepository,
        households::HouseholdRepository,
    },
    pagination::Pagination,
    types::{AccountId, CategoryId, HouseholdId, TransactionId},
    update::FieldUpdate,
};

use super::{
    domain::{
        NewTransaction, Transaction, TransactionAmount, TransactionDetails, TransactionLabel,
        TransactionNature, TransactionNote, TransactionSource, TransactionUpdate,
    },
    repository::{TransactionFilters, TransactionRepository},
};

pub struct CreateTransactionCommand {
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: String,
    pub amount: Decimal,
    pub nature: TransactionNature,
    pub category_id: Option<CategoryId>,
    pub note: Option<String>,
}

pub struct ListTransactionsCommand {
    pub account_id: Option<AccountId>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub nature: Option<TransactionNature>,
    pub category_id: Option<CategoryId>,
    pub source: Option<TransactionSource>,
    pub search: Option<String>,
    pub pagination: Pagination,
}

pub struct UpdateTransactionCommand {
    pub account_id: Option<AccountId>,
    pub booking_date: Option<NaiveDate>,
    pub label: Option<String>,
    pub amount: Option<Decimal>,
    pub nature: Option<TransactionNature>,
    pub category_id: FieldUpdate<CategoryId>,
    pub note: FieldUpdate<String>,
}

impl UpdateTransactionCommand {
    fn is_empty(&self) -> bool {
        self.account_id.is_none()
            && self.booking_date.is_none()
            && self.label.is_none()
            && self.amount.is_none()
            && self.nature.is_none()
            && self.category_id == FieldUpdate::Unchanged
            && self.note == FieldUpdate::Unchanged
    }
}

#[derive(Clone)]
pub struct TransactionService {
    transaction_repository: TransactionRepository,
    household_repository: HouseholdRepository,
    account_repository: AccountRepository,
    category_repository: CategoryRepository,
}

impl TransactionService {
    pub fn new(
        transaction_repository: TransactionRepository,
        household_repository: HouseholdRepository,
        account_repository: AccountRepository,
        category_repository: CategoryRepository,
    ) -> Self {
        Self {
            transaction_repository,
            household_repository,
            account_repository,
            category_repository,
        }
    }

    pub async fn create(
        &self,
        household_id: HouseholdId,
        command: CreateTransactionCommand,
    ) -> Result<Transaction, ApiError> {
        if command.nature == TransactionNature::Transfer {
            return Err(ApiError::BadRequest(
                "transfers must be created through the transfer endpoint".to_owned(),
            ));
        }

        let label = TransactionLabel::new(command.label)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        let amount = TransactionAmount::new(command.amount)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        let note = command
            .note
            .map(TransactionNote::new)
            .transpose()
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;

        self.validate_booking_date(household_id, command.booking_date)
            .await?;
        self.validate_account(household_id, command.account_id)
            .await?;

        self.validate_category(household_id, command.category_id, command.nature)
            .await?;
        let details = match command.nature {
            TransactionNature::Income => TransactionDetails::Income {
                category_id: command.category_id,
            },
            TransactionNature::Expense => TransactionDetails::Expense {
                category_id: command.category_id,
            },
            TransactionNature::Transfer => unreachable!("transfers are rejected above"),
        };

        Ok(self
            .transaction_repository
            .create(NewTransaction {
                id: TransactionId::new(),
                household_id,
                account_id: command.account_id,
                booking_date: command.booking_date,
                label,
                amount,
                details,
                source: TransactionSource::Manual,
                note,
            })
            .await?)
    }

    pub async fn get(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<Transaction, ApiError> {
        self.transaction_repository
            .find(household_id, transaction_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Transaction not found".to_owned()))
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        command: ListTransactionsCommand,
    ) -> Result<Vec<Transaction>, ApiError> {
        if self
            .household_repository
            .find(household_id)
            .await?
            .is_none()
        {
            return Err(ApiError::NotFound("Household not found".to_owned()));
        }
        if matches!(
            (command.date_from, command.date_to),
            (Some(date_from), Some(date_to)) if date_from > date_to
        ) {
            return Err(ApiError::BadRequest(
                "dateFrom must be before or equal to dateTo".to_owned(),
            ));
        }

        let search = command.search.and_then(|search| {
            let search = search.trim();
            (!search.is_empty()).then(|| search.to_owned())
        });
        let filters = TransactionFilters {
            account_id: command.account_id,
            date_from: command.date_from,
            date_to: command.date_to,
            nature: command.nature,
            category_id: command.category_id,
            source: command.source,
            search,
            pagination: command.pagination,
        };

        Ok(self
            .transaction_repository
            .list(household_id, &filters)
            .await?)
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
        command: UpdateTransactionCommand,
    ) -> Result<Transaction, ApiError> {
        let transaction = self.get(household_id, transaction_id).await?;
        if matches!(transaction.details, TransactionDetails::Transfer { .. }) {
            return Err(ApiError::Conflict(
                "transfer movements must be updated through the transfer endpoint".to_owned(),
            ));
        }
        if command.is_empty() {
            return Ok(transaction);
        }
        if transaction.source == TransactionSource::Import
            && (command.account_id.is_some()
                || command.booking_date.is_some()
                || command.label.is_some()
                || command.amount.is_some())
        {
            return Err(ApiError::Conflict(
                "imported transaction bank fields cannot be modified".to_owned(),
            ));
        }

        let account_was_updated = command.account_id.is_some();
        let category_was_updated = !matches!(&command.category_id, FieldUpdate::Unchanged);
        let current_nature = transaction.details.nature();
        let account_id = command.account_id.unwrap_or(transaction.account_id);
        let booking_date = command.booking_date.unwrap_or(transaction.booking_date);
        let label = command
            .label
            .map(TransactionLabel::new)
            .transpose()
            .map_err(|error| ApiError::BadRequest(error.to_string()))?
            .unwrap_or(transaction.label);
        let amount = command
            .amount
            .map(TransactionAmount::new)
            .transpose()
            .map_err(|error| ApiError::BadRequest(error.to_string()))?
            .unwrap_or(transaction.amount);
        let nature = command.nature.unwrap_or(current_nature);
        if nature == TransactionNature::Transfer {
            return Err(ApiError::BadRequest(
                "a regular transaction cannot become a transfer".to_owned(),
            ));
        }
        let category_id = match command.category_id {
            FieldUpdate::Unchanged => transaction.details.category_id(),
            FieldUpdate::Set(category_id) => Some(category_id),
            FieldUpdate::Clear => None,
        };
        let note = match command.note {
            FieldUpdate::Unchanged => transaction.note,
            FieldUpdate::Set(note) => Some(
                TransactionNote::new(note)
                    .map_err(|error| ApiError::BadRequest(error.to_string()))?,
            ),
            FieldUpdate::Clear => None,
        };

        self.validate_booking_date(household_id, booking_date)
            .await?;
        if account_was_updated {
            self.validate_account(household_id, account_id).await?;
        }
        if category_was_updated || nature != current_nature {
            self.validate_category(household_id, category_id, nature)
                .await?;
        }
        let details = match nature {
            TransactionNature::Income => TransactionDetails::Income { category_id },
            TransactionNature::Expense => TransactionDetails::Expense { category_id },
            TransactionNature::Transfer => unreachable!("transfers are rejected above"),
        };

        self.transaction_repository
            .update_regular(TransactionUpdate {
                id: transaction.id,
                household_id,
                account_id,
                booking_date,
                label,
                amount,
                details,
                note,
            })
            .await?
            .ok_or_else(|| ApiError::NotFound("Transaction not found".to_owned()))
    }

    pub async fn delete(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<(), ApiError> {
        let transaction = self.get(household_id, transaction_id).await?;
        if matches!(transaction.details, TransactionDetails::Transfer { .. }) {
            return Err(ApiError::Conflict(
                "transfer movements must be deleted through the transfer endpoint".to_owned(),
            ));
        }
        if self
            .transaction_repository
            .delete_regular(household_id, transaction_id)
            .await?
            == 0
        {
            return Err(ApiError::NotFound("Transaction not found".to_owned()));
        }

        Ok(())
    }

    async fn validate_category(
        &self,
        household_id: HouseholdId,
        category_id: Option<CategoryId>,
        nature: TransactionNature,
    ) -> Result<(), ApiError> {
        let Some(category_id) = category_id else {
            return Ok(());
        };
        let category = self
            .category_repository
            .find(household_id, category_id)
            .await?
            .ok_or_else(|| {
                ApiError::BadRequest(
                    "category must be active and belong to the household".to_owned(),
                )
            })?;
        if category.kind.as_str() != nature.as_str() {
            return Err(ApiError::BadRequest(
                "category kind must match transaction nature".to_owned(),
            ));
        }

        Ok(())
    }

    async fn validate_booking_date(
        &self,
        household_id: HouseholdId,
        booking_date: NaiveDate,
    ) -> Result<(), ApiError> {
        let household = self
            .household_repository
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))?;
        let current_date = household
            .timezone
            .date_at(Utc::now())
            .map_err(|error| ApiError::Database(sqlx::Error::Decode(Box::new(error))))?;
        if booking_date > current_date {
            return Err(ApiError::BadRequest(
                "transaction booking date must not be in the future".to_owned(),
            ));
        }

        Ok(())
    }

    async fn validate_account(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<(), ApiError> {
        if self
            .account_repository
            .find(household_id, account_id)
            .await?
            .is_none()
        {
            return Err(ApiError::BadRequest(
                "account must be active and belong to the household".to_owned(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_update_command() -> UpdateTransactionCommand {
        UpdateTransactionCommand {
            account_id: None,
            booking_date: None,
            label: None,
            amount: None,
            nature: None,
            category_id: FieldUpdate::Unchanged,
            note: FieldUpdate::Unchanged,
        }
    }

    #[test]
    fn unchanged_update_command_is_empty() {
        assert!(empty_update_command().is_empty());
    }

    #[test]
    fn clearing_an_optional_field_is_an_update() {
        let mut command = empty_update_command();
        command.note = FieldUpdate::Clear;

        assert!(!command.is_empty());
    }
}
