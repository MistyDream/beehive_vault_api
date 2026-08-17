use std::error::Error;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    database::Database,
    pagination::Pagination,
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
};

use super::domain::{
    NewTransaction, Transaction, TransactionAmount, TransactionDetails, TransactionLabel,
    TransactionNature, TransactionNote, TransactionSource, TransactionUpdate, TransferRole,
};

#[derive(Debug, Clone)]
pub struct TransactionFilters {
    pub account_id: Option<AccountId>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub nature: Option<TransactionNature>,
    pub category_id: Option<CategoryId>,
    pub source: Option<TransactionSource>,
    pub search: Option<String>,
    pub pagination: Pagination,
}

#[derive(Clone)]
pub struct TransactionRepository {
    database: Database,
}

impl TransactionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, transaction: NewTransaction) -> Result<Transaction, sqlx::Error> {
        let row = sqlx::query_as::<_, TransactionRow>(
            "INSERT INTO transactions \
             (id, household_id, account_id, booking_date, label, amount, nature, category_id, \
              transfer_id, transfer_role, note, source) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             RETURNING id, household_id, account_id, booking_date, label, amount, nature, \
             category_id, transfer_id, transfer_role, note, source, created_at, updated_at, \
             deleted_at",
        )
        .bind(transaction.id)
        .bind(transaction.household_id)
        .bind(transaction.account_id)
        .bind(transaction.booking_date)
        .bind(transaction.label.as_str())
        .bind(transaction.amount.value())
        .bind(transaction.details.nature().as_str())
        .bind(transaction.details.category_id())
        .bind(transaction.details.transfer_id())
        .bind(
            transaction
                .details
                .transfer_role()
                .map(TransferRole::as_str),
        )
        .bind(transaction.note.as_ref().map(TransactionNote::as_str))
        .bind(transaction.source.as_str())
        .fetch_one(self.database.pool())
        .await?;

        Transaction::try_from(row)
    }

    pub async fn find(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let row = sqlx::query_as::<_, TransactionRow>(
            "SELECT id, household_id, account_id, booking_date, label, amount, nature, \
             category_id, transfer_id, transfer_role, note, source, created_at, updated_at, \
             deleted_at FROM transactions \
             WHERE household_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(household_id)
        .bind(transaction_id)
        .fetch_optional(self.database.pool())
        .await?;

        row.map(Transaction::try_from).transpose()
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        filters: &TransactionFilters,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TransactionRow>(
            "SELECT id, household_id, account_id, booking_date, label, amount, nature, \
             category_id, transfer_id, transfer_role, note, source, created_at, updated_at, \
             deleted_at FROM transactions \
             WHERE household_id = $1 AND deleted_at IS NULL \
               AND ($2::uuid IS NULL OR account_id = $2) \
               AND ($3::date IS NULL OR booking_date >= $3) \
               AND ($4::date IS NULL OR booking_date <= $4) \
               AND ($5::text IS NULL OR nature = $5) \
               AND ($6::uuid IS NULL OR category_id = $6) \
               AND ($7::text IS NULL OR source = $7) \
               AND ($8::text IS NULL \
                    OR strpos(lower(label), lower($8)) > 0 \
                    OR strpos(lower(COALESCE(note, '')), lower($8)) > 0) \
             ORDER BY booking_date DESC, created_at DESC, id DESC \
             LIMIT $9 OFFSET $10",
        )
        .bind(household_id)
        .bind(filters.account_id)
        .bind(filters.date_from)
        .bind(filters.date_to)
        .bind(filters.nature.map(TransactionNature::as_str))
        .bind(filters.category_id)
        .bind(filters.source.map(TransactionSource::as_str))
        .bind(filters.search.as_deref())
        .bind(filters.pagination.limit())
        .bind(filters.pagination.offset())
        .fetch_all(self.database.pool())
        .await?;

        rows.into_iter().map(Transaction::try_from).collect()
    }

    pub async fn update_regular(
        &self,
        update: TransactionUpdate,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let row = sqlx::query_as::<_, TransactionRow>(
            "UPDATE transactions SET account_id = $3, booking_date = $4, label = $5, \
             amount = $6, nature = $7, category_id = $8, transfer_id = NULL, \
             transfer_role = NULL, note = $9, updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND deleted_at IS NULL \
               AND nature <> 'transfer' AND $7 <> 'transfer' \
               AND (source = 'manual' OR (source = 'import' \
                    AND account_id = $3 AND booking_date = $4 AND label = $5 AND amount = $6)) \
             RETURNING id, household_id, account_id, booking_date, label, amount, nature, \
             category_id, transfer_id, transfer_role, note, source, created_at, updated_at, \
             deleted_at",
        )
        .bind(update.household_id)
        .bind(update.id)
        .bind(update.account_id)
        .bind(update.booking_date)
        .bind(update.label.as_str())
        .bind(update.amount.value())
        .bind(update.details.nature().as_str())
        .bind(update.details.category_id())
        .bind(update.note.as_ref().map(TransactionNote::as_str))
        .fetch_optional(self.database.pool())
        .await?;

        row.map(Transaction::try_from).transpose()
    }

    pub async fn delete_regular(
        &self,
        household_id: HouseholdId,
        transaction_id: TransactionId,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE transactions SET deleted_at = now(), updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND deleted_at IS NULL \
               AND nature <> 'transfer'",
        )
        .bind(household_id)
        .bind(transaction_id)
        .execute(self.database.pool())
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct TransactionRow {
    id: TransactionId,
    household_id: HouseholdId,
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
    nature: String,
    category_id: Option<CategoryId>,
    transfer_id: Option<TransferId>,
    transfer_role: Option<String>,
    note: Option<String>,
    source: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = sqlx::Error;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        let nature = TransactionNature::try_from(row.nature.as_str()).map_err(decode_error)?;
        let details = decode_details(
            nature,
            row.category_id,
            row.transfer_id,
            row.transfer_role.as_deref(),
        )?;
        let note = row
            .note
            .map(TransactionNote::new)
            .transpose()
            .map_err(decode_error)?;

        Ok(Self {
            id: row.id,
            household_id: row.household_id,
            account_id: row.account_id,
            booking_date: row.booking_date,
            label: TransactionLabel::new(row.label).map_err(decode_error)?,
            amount: TransactionAmount::new(row.amount).map_err(decode_error)?,
            details,
            source: TransactionSource::try_from(row.source.as_str()).map_err(decode_error)?,
            note,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }
}

fn decode_details(
    nature: TransactionNature,
    category_id: Option<CategoryId>,
    transfer_id: Option<TransferId>,
    transfer_role: Option<&str>,
) -> Result<TransactionDetails, sqlx::Error> {
    match nature {
        TransactionNature::Income if transfer_id.is_none() && transfer_role.is_none() => {
            Ok(TransactionDetails::Income { category_id })
        }
        TransactionNature::Expense if transfer_id.is_none() && transfer_role.is_none() => {
            Ok(TransactionDetails::Expense { category_id })
        }
        TransactionNature::Transfer => match (category_id, transfer_id, transfer_role) {
            (None, Some(transfer_id), Some(transfer_role)) => Ok(TransactionDetails::Transfer {
                transfer_id,
                role: TransferRole::try_from(transfer_role).map_err(decode_error)?,
            }),
            _ => Err(decode_error(TransactionDetailsDecodeError)),
        },
        TransactionNature::Income | TransactionNature::Expense => {
            Err(decode_error(TransactionDetailsDecodeError))
        }
    }
}

fn decode_error(error: impl Error + Send + Sync + 'static) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(error))
}

#[derive(Debug, thiserror::Error)]
#[error("transaction details stored in database are inconsistent")]
struct TransactionDetailsDecodeError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_details_reconstructs_a_transfer() {
        let transfer_id = TransferId::new();

        let details = decode_details(
            TransactionNature::Transfer,
            None,
            Some(transfer_id),
            Some("source"),
        )
        .unwrap();

        assert_eq!(
            details,
            TransactionDetails::Transfer {
                transfer_id,
                role: TransferRole::Source,
            }
        );
    }

    #[test]
    fn decode_details_rejects_inconsistent_columns() {
        assert!(
            decode_details(
                TransactionNature::Income,
                None,
                Some(TransferId::new()),
                Some("source"),
            )
            .is_err()
        );
        assert!(
            decode_details(TransactionNature::Transfer, None, None, Some("destination")).is_err()
        );
    }
}
