use std::error::Error;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgConnection;

use crate::{
    database::Database,
    features::transactions::domain::{
        TransactionAmount, TransactionLabel, TransactionNote, TransferRole,
    },
    pagination::Pagination,
    types::{AccountId, HouseholdId, TransactionId, TransferId},
};

use super::domain::{NewTransfer, NewTransferMovement, Transfer, TransferMovement, TransferUpdate};

#[derive(Clone)]
pub struct TransferRepository {
    database: Database,
}

impl TransferRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, transfer: NewTransfer) -> Result<Transfer, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let metadata =
            Self::insert_transfer(&mut transaction, transfer.id, transfer.household_id).await?;
        let source = Self::insert_movement(
            &mut transaction,
            transfer.household_id,
            transfer.id,
            TransferRole::Source,
            transfer.source,
        )
        .await?;
        let destination = Self::insert_movement(
            &mut transaction,
            transfer.household_id,
            transfer.id,
            TransferRole::Destination,
            transfer.destination,
        )
        .await?;
        transaction.commit().await?;

        Ok(metadata.into_transfer(source, destination))
    }

    async fn insert_transfer(
        connection: &mut PgConnection,
        transfer_id: TransferId,
        household_id: HouseholdId,
    ) -> Result<TransferMetadataRow, sqlx::Error> {
        sqlx::query_as::<_, TransferMetadataRow>(
            "INSERT INTO transfers (id, household_id) VALUES ($1, $2) \
             RETURNING id, household_id, created_at, updated_at, deleted_at",
        )
        .bind(transfer_id)
        .bind(household_id)
        .fetch_one(connection)
        .await
    }

    async fn insert_movement(
        connection: &mut PgConnection,
        household_id: HouseholdId,
        transfer_id: TransferId,
        role: TransferRole,
        movement: NewTransferMovement,
    ) -> Result<TransferMovement, sqlx::Error> {
        let row = sqlx::query_as::<_, TransferMovementRow>(
            "INSERT INTO transactions \
             (id, household_id, account_id, booking_date, label, amount, nature, transfer_id, \
              transfer_role, note, source) \
             VALUES ($1, $2, $3, $4, $5, $6, 'transfer', $7, $8, $9, 'manual') \
             RETURNING id AS transaction_id, account_id, booking_date, label, amount, note",
        )
        .bind(movement.transaction_id)
        .bind(household_id)
        .bind(movement.account_id)
        .bind(movement.booking_date)
        .bind(movement.label.as_str())
        .bind(movement.amount.value())
        .bind(transfer_id)
        .bind(role.as_str())
        .bind(movement.note.as_ref().map(TransactionNote::as_str))
        .fetch_one(connection)
        .await?;

        TransferMovement::try_from(row)
    }

    pub async fn find(
        &self,
        household_id: HouseholdId,
        transfer_id: TransferId,
    ) -> Result<Option<Transfer>, sqlx::Error> {
        let row = sqlx::query_as::<_, TransferRow>(
            "SELECT t.id, t.household_id, t.created_at, t.updated_at, t.deleted_at, \
             source.id AS source_transaction_id, source.account_id AS source_account_id, \
             source.booking_date AS source_booking_date, source.label AS source_label, \
             source.amount AS source_amount, source.note AS source_note, \
             destination.id AS destination_transaction_id, \
             destination.account_id AS destination_account_id, \
             destination.booking_date AS destination_booking_date, \
             destination.label AS destination_label, destination.amount AS destination_amount, \
             destination.note AS destination_note \
             FROM transfers t \
             JOIN transactions source ON source.transfer_id = t.id \
               AND source.transfer_role = 'source' AND source.deleted_at IS NULL \
             JOIN transactions destination ON destination.transfer_id = t.id \
               AND destination.transfer_role = 'destination' AND destination.deleted_at IS NULL \
             WHERE t.household_id = $1 AND t.id = $2 AND t.deleted_at IS NULL",
        )
        .bind(household_id)
        .bind(transfer_id)
        .fetch_optional(self.database.pool())
        .await?;

        row.map(Transfer::try_from).transpose()
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        pagination: Pagination,
    ) -> Result<Vec<Transfer>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TransferRow>(
            "SELECT t.id, t.household_id, t.created_at, t.updated_at, t.deleted_at, \
             source.id AS source_transaction_id, source.account_id AS source_account_id, \
             source.booking_date AS source_booking_date, source.label AS source_label, \
             source.amount AS source_amount, source.note AS source_note, \
             destination.id AS destination_transaction_id, \
             destination.account_id AS destination_account_id, \
             destination.booking_date AS destination_booking_date, \
             destination.label AS destination_label, destination.amount AS destination_amount, \
             destination.note AS destination_note \
             FROM transfers t \
             JOIN transactions source ON source.transfer_id = t.id \
               AND source.transfer_role = 'source' AND source.deleted_at IS NULL \
             JOIN transactions destination ON destination.transfer_id = t.id \
               AND destination.transfer_role = 'destination' AND destination.deleted_at IS NULL \
             WHERE t.household_id = $1 AND t.deleted_at IS NULL \
             ORDER BY GREATEST(source.booking_date, destination.booking_date) DESC, \
               t.created_at DESC, t.id DESC LIMIT $2 OFFSET $3",
        )
        .bind(household_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(self.database.pool())
        .await?;

        rows.into_iter().map(Transfer::try_from).collect()
    }

    pub async fn update(&self, update: TransferUpdate) -> Result<Option<Transfer>, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let Some(metadata) =
            Self::update_transfer(&mut transaction, update.household_id, update.id).await?
        else {
            return Ok(None);
        };
        let source = Self::update_movement(
            &mut transaction,
            update.household_id,
            update.id,
            TransferRole::Source,
            update.source,
        )
        .await?;
        let destination = Self::update_movement(
            &mut transaction,
            update.household_id,
            update.id,
            TransferRole::Destination,
            update.destination,
        )
        .await?;
        transaction.commit().await?;

        Ok(Some(metadata.into_transfer(source, destination)))
    }

    async fn update_transfer(
        connection: &mut PgConnection,
        household_id: HouseholdId,
        transfer_id: TransferId,
    ) -> Result<Option<TransferMetadataRow>, sqlx::Error> {
        sqlx::query_as::<_, TransferMetadataRow>(
            "UPDATE transfers SET updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, household_id, created_at, updated_at, deleted_at",
        )
        .bind(household_id)
        .bind(transfer_id)
        .fetch_optional(connection)
        .await
    }

    async fn update_movement(
        connection: &mut PgConnection,
        household_id: HouseholdId,
        transfer_id: TransferId,
        role: TransferRole,
        movement: NewTransferMovement,
    ) -> Result<TransferMovement, sqlx::Error> {
        let row = sqlx::query_as::<_, TransferMovementRow>(
            "UPDATE transactions SET account_id = $4, booking_date = $5, label = $6, \
             amount = $7, note = $8, updated_at = now() \
             WHERE household_id = $1 AND transfer_id = $2 AND transfer_role = $3 \
               AND id = $9 AND nature = 'transfer' AND deleted_at IS NULL \
             RETURNING id AS transaction_id, account_id, booking_date, label, amount, note",
        )
        .bind(household_id)
        .bind(transfer_id)
        .bind(role.as_str())
        .bind(movement.account_id)
        .bind(movement.booking_date)
        .bind(movement.label.as_str())
        .bind(movement.amount.value())
        .bind(movement.note.as_ref().map(TransactionNote::as_str))
        .bind(movement.transaction_id)
        .fetch_optional(connection)
        .await?
        .ok_or_else(|| decode_error(TransferPairDecodeError))?;

        TransferMovement::try_from(row)
    }

    pub async fn delete(
        &self,
        household_id: HouseholdId,
        transfer_id: TransferId,
    ) -> Result<u64, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let transfer_affected = sqlx::query(
            "UPDATE transfers SET deleted_at = now(), updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(household_id)
        .bind(transfer_id)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
        if transfer_affected == 0 {
            return Ok(0);
        }

        let movement_affected = sqlx::query(
            "UPDATE transactions SET deleted_at = now(), updated_at = now() \
             WHERE household_id = $1 AND transfer_id = $2 AND deleted_at IS NULL",
        )
        .bind(household_id)
        .bind(transfer_id)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
        if movement_affected != 2 {
            return Err(decode_error(TransferPairDecodeError));
        }
        transaction.commit().await?;

        Ok(movement_affected)
    }
}

#[derive(sqlx::FromRow)]
struct TransferMetadataRow {
    id: TransferId,
    household_id: HouseholdId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl TransferMetadataRow {
    fn into_transfer(self, source: TransferMovement, destination: TransferMovement) -> Transfer {
        Transfer {
            id: self.id,
            household_id: self.household_id,
            source,
            destination,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TransferMovementRow {
    transaction_id: TransactionId,
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
    note: Option<String>,
}

impl TryFrom<TransferMovementRow> for TransferMovement {
    type Error = sqlx::Error;

    fn try_from(row: TransferMovementRow) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_id: row.transaction_id,
            account_id: row.account_id,
            booking_date: row.booking_date,
            label: TransactionLabel::new(row.label).map_err(decode_error)?,
            amount: TransactionAmount::new(row.amount).map_err(decode_error)?,
            note: row
                .note
                .map(TransactionNote::new)
                .transpose()
                .map_err(decode_error)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TransferRow {
    id: TransferId,
    household_id: HouseholdId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    source_transaction_id: TransactionId,
    source_account_id: AccountId,
    source_booking_date: NaiveDate,
    source_label: String,
    source_amount: Decimal,
    source_note: Option<String>,
    destination_transaction_id: TransactionId,
    destination_account_id: AccountId,
    destination_booking_date: NaiveDate,
    destination_label: String,
    destination_amount: Decimal,
    destination_note: Option<String>,
}

impl TryFrom<TransferRow> for Transfer {
    type Error = sqlx::Error;

    fn try_from(row: TransferRow) -> Result<Self, Self::Error> {
        let source = TransferMovement::try_from(TransferMovementRow {
            transaction_id: row.source_transaction_id,
            account_id: row.source_account_id,
            booking_date: row.source_booking_date,
            label: row.source_label,
            amount: row.source_amount,
            note: row.source_note,
        })?;
        let destination = TransferMovement::try_from(TransferMovementRow {
            transaction_id: row.destination_transaction_id,
            account_id: row.destination_account_id,
            booking_date: row.destination_booking_date,
            label: row.destination_label,
            amount: row.destination_amount,
            note: row.destination_note,
        })?;
        if source.amount.value().abs() != destination.amount.value().abs() {
            return Err(decode_error(TransferPairDecodeError));
        }

        Ok(Self {
            id: row.id,
            household_id: row.household_id,
            source,
            destination,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }
}

fn decode_error(error: impl Error + Send + Sync + 'static) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(error))
}

#[derive(Debug, thiserror::Error)]
#[error("transfer must contain exactly one active source and destination movement")]
struct TransferPairDecodeError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_row_reconstructs_both_movements() {
        let transfer_id = TransferId::new();
        let source_account_id = AccountId::new();
        let destination_account_id = AccountId::new();
        let transfer = Transfer::try_from(TransferRow {
            id: transfer_id,
            household_id: HouseholdId::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            source_transaction_id: TransactionId::new(),
            source_account_id,
            source_booking_date: NaiveDate::from_ymd_opt(2026, 8, 17).unwrap(),
            source_label: "Transfer to savings".to_owned(),
            source_amount: Decimal::new(-500, 0),
            source_note: None,
            destination_transaction_id: TransactionId::new(),
            destination_account_id,
            destination_booking_date: NaiveDate::from_ymd_opt(2026, 8, 18).unwrap(),
            destination_label: "Transfer from checking".to_owned(),
            destination_amount: Decimal::new(500, 0),
            destination_note: Some("Monthly savings".to_owned()),
        })
        .unwrap();

        assert_eq!(transfer.id, transfer_id);
        assert_eq!(transfer.source.account_id, source_account_id);
        assert_eq!(transfer.destination.account_id, destination_account_id);
        assert_eq!(
            transfer.destination.note.unwrap().as_str(),
            "Monthly savings"
        );
    }
}
