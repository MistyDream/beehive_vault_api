use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    error::{ApiError, ProblemKind},
    features::{
        accounts::{Account, AccountRepository},
        households::HouseholdRepository,
        transactions::domain::{
            TransactionAmount, TransactionLabel, TransactionNote, TransferRole,
        },
    },
    pagination::Pagination,
    types::{AccountId, HouseholdId, TransactionId, TransferId},
    update::FieldUpdate,
};

use super::{
    domain::{NewTransfer, NewTransferMovement, Transfer, TransferAmount, TransferUpdate},
    repository::TransferRepository,
};

pub struct TransferMovementCommand {
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: String,
    pub note: Option<String>,
}

pub struct CreateTransferCommand {
    pub amount: Decimal,
    pub source: TransferMovementCommand,
    pub destination: TransferMovementCommand,
}

#[derive(Default)]
pub struct UpdateTransferMovementCommand {
    pub account_id: Option<AccountId>,
    pub booking_date: Option<NaiveDate>,
    pub label: Option<String>,
    pub note: FieldUpdate<String>,
}

impl UpdateTransferMovementCommand {
    fn is_empty(&self) -> bool {
        self.account_id.is_none()
            && self.booking_date.is_none()
            && self.label.is_none()
            && self.note == FieldUpdate::Unchanged
    }
}

pub struct UpdateTransferCommand {
    pub amount: Option<Decimal>,
    pub source: Option<UpdateTransferMovementCommand>,
    pub destination: Option<UpdateTransferMovementCommand>,
}

impl UpdateTransferCommand {
    fn is_empty(&self) -> bool {
        self.amount.is_none()
            && self
                .source
                .as_ref()
                .is_none_or(UpdateTransferMovementCommand::is_empty)
            && self
                .destination
                .as_ref()
                .is_none_or(UpdateTransferMovementCommand::is_empty)
    }
}

#[derive(Clone)]
pub struct TransferService {
    transfer_repository: TransferRepository,
    household_repository: HouseholdRepository,
    account_repository: AccountRepository,
}

impl TransferService {
    pub fn new(
        transfer_repository: TransferRepository,
        household_repository: HouseholdRepository,
        account_repository: AccountRepository,
    ) -> Self {
        Self {
            transfer_repository,
            household_repository,
            account_repository,
        }
    }

    pub async fn create(
        &self,
        household_id: HouseholdId,
        command: CreateTransferCommand,
    ) -> Result<Transfer, ApiError> {
        if command.source.account_id == command.destination.account_id {
            return Err(ApiError::body_validation(
                "#/destination/accountId",
                "transfer_accounts_must_differ",
                "source and destination accounts must be different",
            ));
        }

        let amount = TransferAmount::new(command.amount).map_err(|error| {
            ApiError::body_validation("#/amount", "non_positive_amount", error.to_string())
        })?;
        let current_date = self.current_date(household_id).await?;
        self.validate_date(
            command.source.booking_date,
            current_date,
            "#/source/bookingDate",
        )?;
        self.validate_date(
            command.destination.booking_date,
            current_date,
            "#/destination/bookingDate",
        )?;
        let source_account = self
            .active_account(
                household_id,
                command.source.account_id,
                "#/source/accountId",
            )
            .await?;
        let destination_account = self
            .active_account(
                household_id,
                command.destination.account_id,
                "#/destination/accountId",
            )
            .await?;

        let source =
            Self::create_movement(command.source, amount, TransferRole::Source, source_account)?;
        let destination = Self::create_movement(
            command.destination,
            amount,
            TransferRole::Destination,
            destination_account,
        )?;

        Ok(self
            .transfer_repository
            .create(NewTransfer {
                id: TransferId::new(),
                household_id,
                source,
                destination,
            })
            .await?)
    }

    fn create_movement(
        command: TransferMovementCommand,
        amount: TransferAmount,
        role: TransferRole,
        account: Account,
    ) -> Result<NewTransferMovement, ApiError> {
        let pointer_prefix = format!("#/{}", role.as_str());
        Ok(NewTransferMovement {
            transaction_id: TransactionId::new(),
            account_id: command.account_id,
            booking_date: command.booking_date,
            label: TransactionLabel::new(command.label).map_err(|error| {
                ApiError::body_validation(
                    format!("{pointer_prefix}/label"),
                    "invalid_length",
                    error.to_string(),
                )
            })?,
            amount: TransactionAmount::new(amount.signed_value(role, account.kind)).map_err(
                |error| ApiError::body_validation("#/amount", "zero_amount", error.to_string()),
            )?,
            note: command
                .note
                .map(TransactionNote::new)
                .transpose()
                .map_err(|error| {
                    ApiError::body_validation(
                        format!("{pointer_prefix}/note"),
                        "invalid_length",
                        error.to_string(),
                    )
                })?,
        })
    }

    pub async fn get(
        &self,
        household_id: HouseholdId,
        transfer_id: TransferId,
    ) -> Result<Transfer, ApiError> {
        self.transfer_repository
            .find(household_id, transfer_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::TransferNotFound))
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        pagination: Pagination,
    ) -> Result<Vec<Transfer>, ApiError> {
        if self
            .household_repository
            .find(household_id)
            .await?
            .is_none()
        {
            return Err(ApiError::new(ProblemKind::HouseholdNotFound));
        }

        Ok(self
            .transfer_repository
            .list(household_id, pagination)
            .await?)
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        transfer_id: TransferId,
        command: UpdateTransferCommand,
    ) -> Result<Transfer, ApiError> {
        let transfer = self.get(household_id, transfer_id).await?;
        if command.is_empty() {
            return Ok(transfer);
        }

        let amount = command
            .amount
            .map(TransferAmount::new)
            .transpose()
            .map_err(|error| {
                ApiError::body_validation("#/amount", "non_positive_amount", error.to_string())
            })?
            .unwrap_or_else(|| transfer.amount());
        let source_update = command.source.unwrap_or_default();
        let destination_update = command.destination.unwrap_or_default();
        let source_account = self
            .resolve_account(
                household_id,
                transfer.source.account_id,
                source_update.account_id,
                "#/source/accountId",
            )
            .await?;
        let destination_account = self
            .resolve_account(
                household_id,
                transfer.destination.account_id,
                destination_update.account_id,
                "#/destination/accountId",
            )
            .await?;
        if source_account.id == destination_account.id {
            return Err(ApiError::body_validation(
                "#/destination/accountId",
                "transfer_accounts_must_differ",
                "source and destination accounts must be different",
            ));
        }

        let source = Self::update_movement(
            transfer.source,
            source_update,
            amount,
            TransferRole::Source,
            source_account,
        )?;
        let destination = Self::update_movement(
            transfer.destination,
            destination_update,
            amount,
            TransferRole::Destination,
            destination_account,
        )?;
        let current_date = self.current_date(household_id).await?;
        self.validate_date(source.booking_date, current_date, "#/source/bookingDate")?;
        self.validate_date(
            destination.booking_date,
            current_date,
            "#/destination/bookingDate",
        )?;

        self.transfer_repository
            .update(TransferUpdate {
                id: transfer.id,
                household_id,
                source,
                destination,
            })
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::TransferNotFound))
    }

    fn update_movement(
        current: super::domain::TransferMovement,
        update: UpdateTransferMovementCommand,
        amount: TransferAmount,
        role: TransferRole,
        account: Account,
    ) -> Result<NewTransferMovement, ApiError> {
        let pointer_prefix = format!("#/{}", role.as_str());
        let label = update
            .label
            .map(TransactionLabel::new)
            .transpose()
            .map_err(|error| {
                ApiError::body_validation(
                    format!("{pointer_prefix}/label"),
                    "invalid_length",
                    error.to_string(),
                )
            })?
            .unwrap_or(current.label);
        let note = match update.note {
            FieldUpdate::Unchanged => current.note,
            FieldUpdate::Set(note) => Some(TransactionNote::new(note).map_err(|error| {
                ApiError::body_validation(
                    format!("{pointer_prefix}/note"),
                    "invalid_length",
                    error.to_string(),
                )
            })?),
            FieldUpdate::Clear => None,
        };

        Ok(NewTransferMovement {
            transaction_id: current.transaction_id,
            account_id: account.id,
            booking_date: update.booking_date.unwrap_or(current.booking_date),
            label,
            amount: TransactionAmount::new(amount.signed_value(role, account.kind)).map_err(
                |error| ApiError::body_validation("#/amount", "zero_amount", error.to_string()),
            )?,
            note,
        })
    }

    pub async fn delete(
        &self,
        household_id: HouseholdId,
        transfer_id: TransferId,
    ) -> Result<(), ApiError> {
        if self
            .transfer_repository
            .delete(household_id, transfer_id)
            .await?
            == 0
        {
            return Err(ApiError::new(ProblemKind::TransferNotFound));
        }

        Ok(())
    }

    async fn active_account(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        pointer: &'static str,
    ) -> Result<Account, ApiError> {
        self.account_repository
            .find(household_id, account_id)
            .await?
            .ok_or_else(|| {
                ApiError::body_validation(
                    pointer,
                    "invalid_account",
                    "account must be active and belong to the household",
                )
            })
    }

    async fn resolve_account(
        &self,
        household_id: HouseholdId,
        current_account_id: AccountId,
        requested_account_id: Option<AccountId>,
        pointer: &'static str,
    ) -> Result<Account, ApiError> {
        if let Some(account_id) = requested_account_id {
            return self.active_account(household_id, account_id, pointer).await;
        }

        self.account_repository
            .find_including_archived(household_id, current_account_id)
            .await?
            .ok_or_else(|| {
                tracing::error!(%household_id, %current_account_id, "transfer account is missing");
                ApiError::new(ProblemKind::InternalError)
            })
    }

    async fn current_date(&self, household_id: HouseholdId) -> Result<NaiveDate, ApiError> {
        let household = self
            .household_repository
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;

        household
            .timezone
            .date_at(Utc::now())
            .map_err(|error| ApiError::from(sqlx::Error::Decode(Box::new(error))))
    }

    fn validate_date(
        &self,
        booking_date: NaiveDate,
        current_date: NaiveDate,
        pointer: &'static str,
    ) -> Result<(), ApiError> {
        if booking_date > current_date {
            return Err(ApiError::body_validation(
                pointer,
                "date_in_future",
                "transfer booking dates must not be in the future",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_update_command() -> UpdateTransferCommand {
        UpdateTransferCommand {
            amount: None,
            source: None,
            destination: None,
        }
    }

    #[test]
    fn unchanged_update_command_is_empty() {
        assert!(empty_update_command().is_empty());
    }

    #[test]
    fn nested_empty_movement_update_is_empty() {
        let mut command = empty_update_command();
        command.source = Some(UpdateTransferMovementCommand::default());

        assert!(command.is_empty());
    }

    #[test]
    fn clearing_a_movement_note_is_an_update() {
        let mut command = empty_update_command();
        command.destination = Some(UpdateTransferMovementCommand {
            note: FieldUpdate::Clear,
            ..Default::default()
        });

        assert!(!command.is_empty());
    }
}
