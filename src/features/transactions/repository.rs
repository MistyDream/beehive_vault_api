use std::error::Error;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    database::Database,
    features::{accounts::AccountKind, categories::CategoryKind},
    pagination::Pagination,
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
};

use super::domain::{
    NewTransaction, Transaction, TransactionAmount, TransactionAmounts, TransactionDetails,
    TransactionLabel, TransactionNature, TransactionNominalAmount, TransactionNote,
    TransactionSource, TransactionUpdate, TransferRole,
};
use super::operations::{
    AccountSummary, CategorySummary, Operation, TransactionOperation, TransferMovementOperation,
    TransferOperation,
};

#[derive(Debug, Clone)]
pub struct TransactionFilters {
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

    pub async fn list_operations(
        &self,
        household_id: HouseholdId,
        filters: &TransactionFilters,
    ) -> Result<(Vec<Operation>, i64), sqlx::Error> {
        let rows = sqlx::query_as::<_, OperationRow>(
            "WITH matching_operations AS ( \
             SELECT 'transaction'::text AS operation_type, transaction.id AS operation_id, \
                    transaction.booking_date, transaction.created_at, transaction.updated_at \
             FROM transactions AS transaction \
             WHERE transaction.household_id = $1 AND transaction.deleted_at IS NULL \
               AND transaction.transfer_id IS NULL \
               AND ($2::uuid IS NULL OR transaction.account_id = $2) \
               AND ($3::date IS NULL OR transaction.booking_date >= $3) \
               AND ($4::date IS NULL OR transaction.booking_date <= $4) \
               AND ($5::text IS NULL OR transaction.nature = $5) \
               AND ($6::uuid IS NULL OR transaction.category_id = $6) \
               AND (NOT $7 OR transaction.category_id IS NULL) \
               AND ($8::text IS NULL OR transaction.source = $8) \
               AND ($9::text IS NULL \
                    OR strpos(lower(transaction.label), lower($9)) > 0 \
                    OR strpos(lower(COALESCE(transaction.note, '')), lower($9)) > 0) \
             UNION ALL \
             SELECT 'transfer'::text, transfer.id, source.booking_date, \
                    transfer.created_at, transfer.updated_at \
             FROM transfers AS transfer \
             JOIN transactions AS source ON source.transfer_id = transfer.id \
               AND source.transfer_role = 'source' AND source.deleted_at IS NULL \
             JOIN transactions AS destination ON destination.transfer_id = transfer.id \
               AND destination.transfer_role = 'destination' AND destination.deleted_at IS NULL \
             WHERE transfer.household_id = $1 AND transfer.deleted_at IS NULL \
               AND ($2::uuid IS NULL \
                    OR source.account_id = $2 OR destination.account_id = $2) \
               AND ($3::date IS NULL OR source.booking_date >= $3) \
               AND ($4::date IS NULL OR source.booking_date <= $4) \
               AND ($5::text IS NULL OR $5 = 'transfer') \
               AND $6::uuid IS NULL AND NOT $7 \
               AND ($8::text IS NULL OR $8 = 'manual') \
               AND ($9::text IS NULL \
                    OR strpos(lower(source.label), lower($9)) > 0 \
                    OR strpos(lower(COALESCE(source.note, '')), lower($9)) > 0 \
                    OR strpos(lower(destination.label), lower($9)) > 0 \
                    OR strpos(lower(COALESCE(destination.note, '')), lower($9)) > 0) \
             ) \
             SELECT operation_total.total, operation.operation_type, operation.operation_id, \
                    operation.booking_date, operation.created_at, operation.updated_at, \
                    COALESCE(transaction.household_id, transfer.household_id) AS household_id, \
                    transaction.label AS transaction_label, \
                    transaction.amount AS transaction_amount, \
                    transaction.nature AS transaction_nature, \
                    transaction.source AS transaction_origin, \
                    transaction.note AS transaction_note, \
                    transaction_account.id AS transaction_account_id, \
                    transaction_account.name AS transaction_account_name, \
                    transaction_account.kind AS transaction_account_kind, \
                    transaction_account.archived_at IS NOT NULL AS transaction_account_archived, \
                    category.id AS category_id, category.name AS category_name, \
                    category.kind AS category_kind, \
                    category.archived_at IS NOT NULL AS category_archived, \
                    source.id AS source_transaction_id, \
                    source.booking_date AS source_booking_date, source.label AS source_label, \
                    source.amount AS source_account_amount, source.note AS source_note, \
                    source_account.id AS source_account_id, \
                    source_account.name AS source_account_name, \
                    source_account.kind AS source_account_kind, \
                    source_account.archived_at IS NOT NULL AS source_account_archived, \
                    destination.id AS destination_transaction_id, \
                    destination.booking_date AS destination_booking_date, \
                    destination.label AS destination_label, \
                    destination.amount AS destination_account_amount, \
                    destination.note AS destination_note, \
                    destination_account.id AS destination_account_id, \
                    destination_account.name AS destination_account_name, \
                    destination_account.kind AS destination_account_kind, \
                    destination_account.archived_at IS NOT NULL AS destination_account_archived \
             FROM (SELECT count(*) AS total FROM matching_operations) AS operation_total \
             LEFT JOIN LATERAL ( \
               SELECT * FROM matching_operations \
               ORDER BY booking_date DESC, created_at DESC, operation_id DESC \
               LIMIT $10 OFFSET $11 \
             ) AS operation ON true \
             LEFT JOIN transactions AS transaction \
               ON operation.operation_type = 'transaction' \
              AND transaction.id = operation.operation_id \
             LEFT JOIN accounts AS transaction_account \
               ON transaction_account.id = transaction.account_id \
             LEFT JOIN categories AS category ON category.id = transaction.category_id \
             LEFT JOIN transfers AS transfer \
               ON operation.operation_type = 'transfer' AND transfer.id = operation.operation_id \
             LEFT JOIN transactions AS source ON source.transfer_id = transfer.id \
               AND source.transfer_role = 'source' AND source.deleted_at IS NULL \
             LEFT JOIN accounts AS source_account ON source_account.id = source.account_id \
             LEFT JOIN transactions AS destination ON destination.transfer_id = transfer.id \
               AND destination.transfer_role = 'destination' AND destination.deleted_at IS NULL \
             LEFT JOIN accounts AS destination_account \
               ON destination_account.id = destination.account_id \
             ORDER BY operation.booking_date DESC, operation.created_at DESC, \
                      operation.operation_id DESC",
        )
        .bind(household_id)
        .bind(filters.account_id)
        .bind(filters.date_from)
        .bind(filters.date_to)
        .bind(filters.nature.map(TransactionNature::as_str))
        .bind(filters.category_id)
        .bind(filters.uncategorized)
        .bind(filters.source.map(TransactionSource::as_str))
        .bind(filters.search.as_deref())
        .bind(filters.pagination.limit())
        .bind(filters.pagination.offset())
        .fetch_all(self.database.pool())
        .await?;

        let total = rows.first().map_or(0, |row| row.total);
        let operations = rows
            .into_iter()
            .filter_map(|row| row.try_into_operation().transpose())
            .collect::<Result<_, _>>()?;
        Ok((operations, total))
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

#[derive(sqlx::FromRow)]
struct OperationRow {
    total: i64,
    operation_type: Option<String>,
    operation_id: Option<uuid::Uuid>,
    booking_date: Option<NaiveDate>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    household_id: Option<HouseholdId>,
    transaction_label: Option<String>,
    transaction_amount: Option<Decimal>,
    transaction_nature: Option<String>,
    transaction_origin: Option<String>,
    transaction_note: Option<String>,
    transaction_account_id: Option<AccountId>,
    transaction_account_name: Option<String>,
    transaction_account_kind: Option<String>,
    transaction_account_archived: bool,
    category_id: Option<CategoryId>,
    category_name: Option<String>,
    category_kind: Option<String>,
    category_archived: bool,
    source_transaction_id: Option<TransactionId>,
    source_booking_date: Option<NaiveDate>,
    source_label: Option<String>,
    source_account_amount: Option<Decimal>,
    source_note: Option<String>,
    source_account_id: Option<AccountId>,
    source_account_name: Option<String>,
    source_account_kind: Option<String>,
    source_account_archived: bool,
    destination_transaction_id: Option<TransactionId>,
    destination_booking_date: Option<NaiveDate>,
    destination_label: Option<String>,
    destination_account_amount: Option<Decimal>,
    destination_note: Option<String>,
    destination_account_id: Option<AccountId>,
    destination_account_name: Option<String>,
    destination_account_kind: Option<String>,
    destination_account_archived: bool,
}

impl OperationRow {
    fn try_into_operation(mut self) -> Result<Option<Operation>, sqlx::Error> {
        let Some(operation_type) = self.operation_type.take() else {
            return Ok(None);
        };

        match operation_type.as_str() {
            "transaction" => decode_transaction_operation(self)
                .map(Operation::Transaction)
                .map(Some),
            "transfer" => decode_transfer_operation(self)
                .map(Operation::Transfer)
                .map(Some),
            _ => Err(decode_error(OperationDecodeError)),
        }
    }
}

fn decode_transaction_operation(row: OperationRow) -> Result<TransactionOperation, sqlx::Error> {
    let id = row
        .operation_id
        .ok_or_else(|| decode_error(OperationDecodeError))?
        .to_string()
        .parse::<TransactionId>()
        .map_err(decode_error)?;
    let nature = TransactionNature::try_from(required(row.transaction_nature)?.as_str())
        .map_err(decode_error)?;
    let account_kind = AccountKind::try_from(required(row.transaction_account_kind)?.as_str())
        .map_err(decode_error)?;
    let account_amount =
        TransactionAmount::new(required(row.transaction_amount)?).map_err(decode_error)?;
    let category = match row.category_id {
        Some(id) => Some(CategorySummary {
            id,
            name: required(row.category_name)?,
            kind: CategoryKind::try_from(required(row.category_kind)?.as_str())
                .map_err(decode_error)?,
            archived: row.category_archived,
        }),
        None => None,
    };

    Ok(TransactionOperation {
        id,
        household_id: required(row.household_id)?,
        booking_date: required(row.booking_date)?,
        label: TransactionLabel::new(required(row.transaction_label)?).map_err(decode_error)?,
        nature,
        amounts: TransactionAmounts::from_stored(nature, account_amount, account_kind)
            .map_err(decode_error)?,
        account: AccountSummary {
            id: required(row.transaction_account_id)?,
            name: required(row.transaction_account_name)?,
            kind: account_kind,
            archived: row.transaction_account_archived,
        },
        category,
        origin: TransactionSource::try_from(required(row.transaction_origin)?.as_str())
            .map_err(decode_error)?,
        note: row
            .transaction_note
            .map(TransactionNote::new)
            .transpose()
            .map_err(decode_error)?,
        created_at: required(row.created_at)?,
        updated_at: required(row.updated_at)?,
    })
}

fn decode_transfer_operation(row: OperationRow) -> Result<TransferOperation, sqlx::Error> {
    let id = row
        .operation_id
        .ok_or_else(|| decode_error(OperationDecodeError))?
        .to_string()
        .parse::<TransferId>()
        .map_err(decode_error)?;
    let source_amount =
        TransactionAmount::new(required(row.source_account_amount)?).map_err(decode_error)?;
    let destination_amount =
        TransactionAmount::new(required(row.destination_account_amount)?).map_err(decode_error)?;
    let source = TransferMovementOperation {
        transaction_id: required(row.source_transaction_id)?,
        booking_date: required(row.source_booking_date)?,
        label: TransactionLabel::new(required(row.source_label)?).map_err(decode_error)?,
        account_amount: source_amount,
        account: AccountSummary {
            id: required(row.source_account_id)?,
            name: required(row.source_account_name)?,
            kind: AccountKind::try_from(required(row.source_account_kind)?.as_str())
                .map_err(decode_error)?,
            archived: row.source_account_archived,
        },
        note: row
            .source_note
            .map(TransactionNote::new)
            .transpose()
            .map_err(decode_error)?,
    };
    let destination = TransferMovementOperation {
        transaction_id: required(row.destination_transaction_id)?,
        booking_date: required(row.destination_booking_date)?,
        label: TransactionLabel::new(required(row.destination_label)?).map_err(decode_error)?,
        account_amount: destination_amount,
        account: AccountSummary {
            id: required(row.destination_account_id)?,
            name: required(row.destination_account_name)?,
            kind: AccountKind::try_from(required(row.destination_account_kind)?.as_str())
                .map_err(decode_error)?,
            archived: row.destination_account_archived,
        },
        note: row
            .destination_note
            .map(TransactionNote::new)
            .transpose()
            .map_err(decode_error)?,
    };

    Ok(TransferOperation {
        id,
        household_id: required(row.household_id)?,
        booking_date: required(row.booking_date)?,
        amount: TransactionNominalAmount::new(source_amount.value().abs()).map_err(decode_error)?,
        source,
        destination,
        created_at: required(row.created_at)?,
        updated_at: required(row.updated_at)?,
    })
}

fn required<T>(value: Option<T>) -> Result<T, sqlx::Error> {
    value.ok_or_else(|| decode_error(OperationDecodeError))
}

#[derive(Debug, thiserror::Error)]
#[error("operation data stored in database are inconsistent")]
struct OperationDecodeError;

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
