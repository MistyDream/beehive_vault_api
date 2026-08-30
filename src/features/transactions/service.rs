use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    error::{ApiError, ProblemKind},
    features::{
        accounts::{Account, AccountRepository},
        categories::CategoryRepository,
        households::HouseholdRepository,
    },
    pagination::Pagination,
    types::{AccountId, CategoryId, HouseholdId, TransactionId},
    update::FieldUpdate,
};

use super::operations::{AccountSummary, CategorySummary, OperationPage, TransactionOperation};
use super::{
    domain::{
        NewTransaction, Transaction, TransactionAmounts, TransactionDetails, TransactionEffect,
        TransactionLabel, TransactionNature, TransactionNominalAmount, TransactionNote,
        TransactionSource, TransactionUpdate,
    },
    repository::{TransactionFilters, TransactionRepository},
};

pub struct CreateTransactionCommand {
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: String,
    pub amount: Decimal,
    pub effect: TransactionEffect,
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
    pub uncategorized: bool,
    pub source: Option<TransactionSource>,
    pub search: Option<String>,
    pub pagination: Pagination,
}

pub struct UpdateTransactionCommand {
    pub account_id: Option<AccountId>,
    pub booking_date: Option<NaiveDate>,
    pub label: Option<String>,
    pub amount: Option<Decimal>,
    pub effect: Option<TransactionEffect>,
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
            && self.effect.is_none()
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
    ) -> Result<TransactionOperation, ApiError> {
        if command.nature == TransactionNature::Transfer {
            return Err(ApiError::body_validation(
                "#/nature",
                "transfer_endpoint_required",
                "transfers must be created through the transfer endpoint",
            ));
        }

        let label = TransactionLabel::new(command.label).map_err(|error| {
            ApiError::body_validation("#/label", "invalid_length", error.to_string())
        })?;
        let note = command
            .note
            .map(TransactionNote::new)
            .transpose()
            .map_err(|error| {
                ApiError::body_validation("#/note", "invalid_length", error.to_string())
            })?;

        self.validate_booking_date(household_id, command.booking_date)
            .await?;
        let account = self
            .validate_account(household_id, command.account_id)
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
        let nominal = transaction_nominal_amount(command.amount)?;
        let amount =
            TransactionAmounts::from_input(command.nature, command.effect, nominal, account.kind)
                .expect("transfers are rejected before ordinary amount derivation")
                .account;

        let transaction = self
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
            .await?;
        self.present_transaction(transaction).await
    }

    pub async fn get(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<TransactionOperation, ApiError> {
        let transaction = self.get_stored(household_id, transaction_id).await?;
        self.present_transaction(transaction).await
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        command: ListTransactionsCommand,
    ) -> Result<OperationPage, ApiError> {
        if self
            .household_repository
            .find(household_id)
            .await?
            .is_none()
        {
            return Err(ApiError::new(ProblemKind::HouseholdNotFound));
        }
        if matches!(
            (command.date_from, command.date_to),
            (Some(date_from), Some(date_to)) if date_from > date_to
        ) {
            return Err(ApiError::query_validation(
                "#/dateFrom",
                "invalid_date_range",
                "dateFrom must be before or equal to dateTo",
            ));
        }
        if command.uncategorized && command.category_id.is_some() {
            return Err(ApiError::query_validation(
                "#/uncategorized",
                "incompatible_filters",
                "uncategorized cannot be combined with categoryId",
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
            uncategorized: command.uncategorized,
            source: command.source,
            search,
            pagination: command.pagination,
        };

        let (items, total) = self
            .transaction_repository
            .list_operations(household_id, &filters)
            .await?;
        Ok(OperationPage {
            items,
            page: command.pagination.page(),
            limit: command.pagination.limit(),
            total,
        })
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
        command: UpdateTransactionCommand,
    ) -> Result<TransactionOperation, ApiError> {
        let transaction = self.get_stored(household_id, transaction_id).await?;
        if matches!(transaction.details, TransactionDetails::Transfer { .. }) {
            return Err(ApiError::new(ProblemKind::TransferMovementUpdateForbidden)
                .with_detail("Transfer movements must be updated through the transfer endpoint."));
        }
        if command.is_empty() {
            return self.present_transaction(transaction).await;
        }
        if transaction.source == TransactionSource::Import
            && (command.account_id.is_some()
                || command.booking_date.is_some()
                || command.label.is_some()
                || command.amount.is_some()
                || command.effect.is_some())
        {
            return Err(
                ApiError::new(ProblemKind::ImportedTransactionFieldsImmutable)
                    .with_detail("Imported transaction bank fields cannot be modified."),
            );
        }

        let account_was_updated = command.account_id.is_some();
        let category_was_updated = !matches!(&command.category_id, FieldUpdate::Unchanged);
        let current_nature = transaction.details.nature();
        let current_account = self
            .account_repository
            .find_including_archived(household_id, transaction.account_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))?;
        let current_account_kind = current_account.kind;
        let account_id = command.account_id.unwrap_or(transaction.account_id);
        let booking_date = command.booking_date.unwrap_or(transaction.booking_date);
        let label = command
            .label
            .map(TransactionLabel::new)
            .transpose()
            .map_err(|error| {
                ApiError::body_validation("#/label", "invalid_length", error.to_string())
            })?
            .unwrap_or(transaction.label);
        let nature = command.nature.unwrap_or(current_nature);
        if nature == TransactionNature::Transfer {
            return Err(ApiError::body_validation(
                "#/nature",
                "transfer_endpoint_required",
                "a regular transaction cannot become a transfer",
            ));
        }
        let category_id = match command.category_id {
            FieldUpdate::Unchanged => transaction.details.category_id(),
            FieldUpdate::Set(category_id) => Some(category_id),
            FieldUpdate::Clear => None,
        };
        let note = match command.note {
            FieldUpdate::Unchanged => transaction.note,
            FieldUpdate::Set(note) => Some(TransactionNote::new(note).map_err(|error| {
                ApiError::body_validation("#/note", "invalid_length", error.to_string())
            })?),
            FieldUpdate::Clear => None,
        };

        self.validate_booking_date(household_id, booking_date)
            .await?;
        let account = if account_was_updated {
            self.validate_account(household_id, account_id).await?
        } else {
            current_account
        };
        if category_was_updated || nature != current_nature {
            self.validate_category(household_id, category_id, nature)
                .await?;
        }
        let details = match nature {
            TransactionNature::Income => TransactionDetails::Income { category_id },
            TransactionNature::Expense => TransactionDetails::Expense { category_id },
            TransactionNature::Transfer => unreachable!("transfers are rejected above"),
        };
        let current_amounts = TransactionAmounts::from_stored(
            current_nature,
            transaction.amount,
            current_account_kind,
        )
        .expect("a regular transaction always has ordinary amount semantics");
        let nominal = command
            .amount
            .map(transaction_nominal_amount)
            .transpose()?
            .unwrap_or(current_amounts.nominal);
        let effect = command.effect.unwrap_or(current_amounts.effect);
        let amount = TransactionAmounts::from_input(nature, effect, nominal, account.kind)
            .expect("transfers are rejected before ordinary amount derivation")
            .account;

        let transaction = self
            .transaction_repository
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
            .ok_or_else(|| ApiError::new(ProblemKind::TransactionNotFound))?;
        self.present_transaction(transaction).await
    }

    pub async fn delete(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<(), ApiError> {
        let transaction = self.get_stored(household_id, transaction_id).await?;
        if matches!(transaction.details, TransactionDetails::Transfer { .. }) {
            return Err(ApiError::new(ProblemKind::TransferMovementDeleteForbidden)
                .with_detail("Transfer movements must be deleted through the transfer endpoint."));
        }
        if self
            .transaction_repository
            .delete_regular(household_id, transaction_id)
            .await?
            == 0
        {
            return Err(ApiError::new(ProblemKind::TransactionNotFound));
        }

        Ok(())
    }

    async fn get_stored(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<Transaction, ApiError> {
        self.transaction_repository
            .find(household_id, transaction_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::TransactionNotFound))
    }

    async fn present_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionOperation, ApiError> {
        let nature = transaction.details.nature();
        if nature == TransactionNature::Transfer {
            return Err(ApiError::new(ProblemKind::TransactionNotFound));
        }
        let account = self
            .account_repository
            .find_including_archived(transaction.household_id, transaction.account_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))?;
        let category = match transaction.details.category_id() {
            Some(category_id) => {
                let category = self
                    .category_repository
                    .find_including_archived(transaction.household_id, category_id)
                    .await?
                    .ok_or_else(|| ApiError::new(ProblemKind::CategoryNotFound))?;
                Some(CategorySummary {
                    id: category.id,
                    name: category.name.into_string(),
                    kind: category.kind,
                    archived: category.archived_at.is_some(),
                })
            }
            None => None,
        };
        let amounts = TransactionAmounts::from_stored(nature, transaction.amount, account.kind)
            .expect("a regular transaction always has ordinary amount semantics");

        Ok(TransactionOperation {
            id: transaction.id,
            household_id: transaction.household_id,
            booking_date: transaction.booking_date,
            label: transaction.label,
            nature,
            amounts,
            account: AccountSummary {
                id: account.id,
                name: account.name,
                kind: account.kind,
                archived: account.archived_at.is_some(),
            },
            category,
            origin: transaction.source,
            note: transaction.note,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        })
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
                ApiError::body_validation(
                    "#/categoryId",
                    "invalid_category",
                    "category must be active and belong to the household",
                )
            })?;
        if category.kind.as_str() != nature.as_str() {
            return Err(ApiError::body_validation(
                "#/categoryId",
                "category_kind_mismatch",
                "category kind must match transaction nature",
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
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        let current_date = household
            .timezone
            .date_at(Utc::now())
            .map_err(|error| ApiError::from(sqlx::Error::Decode(Box::new(error))))?;
        if booking_date > current_date {
            return Err(ApiError::body_validation(
                "#/bookingDate",
                "date_in_future",
                "transaction booking date must not be in the future",
            ));
        }

        Ok(())
    }

    async fn validate_account(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Account, ApiError> {
        self.account_repository
            .find(household_id, account_id)
            .await?
            .ok_or_else(|| {
                ApiError::body_validation(
                    "#/accountId",
                    "invalid_account",
                    "account must be active and belong to the household",
                )
            })
    }
}

fn transaction_nominal_amount(amount: Decimal) -> Result<TransactionNominalAmount, ApiError> {
    TransactionNominalAmount::new(amount)
        .map_err(|error| ApiError::body_validation("#/amount", "invalid_amount", error.to_string()))
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
            effect: None,
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
