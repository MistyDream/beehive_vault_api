use crate::{
    database::Database,
    features::{
        accounts::AccountRepository,
        transactions::domain::{TransactionNature, TransactionSource},
    },
    types::{AccountId, HouseholdId, TransactionId},
};

use super::domain::{ImportStatus, ImportedRow, SelectedRow};

#[derive(Clone)]
pub(super) struct CsvImportRepository {
    database: Database,
}

impl CsvImportRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn confirm(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        statement_digest: &[u8],
        file_name: &str,
        rows: Vec<SelectedRow<'_>>,
    ) -> Result<Option<Vec<ImportedRow>>, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let Some(account_kind) =
            AccountRepository::lock_active_kind(&mut transaction, household_id, account_id).await?
        else {
            return Ok(None);
        };

        let mut outcomes = Vec::with_capacity(rows.len());
        for row in rows {
            let line = row.selection.line;
            let nature: TransactionNature = row.selection.nature.into();
            let inserted = sqlx::query_scalar::<_, TransactionId>(
                "INSERT INTO transactions \
                 (id, household_id, account_id, booking_date, label, amount, nature, source, \
                  csv_statement_digest, csv_source_line, csv_file_name, csv_signed_amount, csv_source_record) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
                 ON CONFLICT (account_id, csv_statement_digest, csv_source_line) \
                 WHERE csv_statement_digest IS NOT NULL DO NOTHING RETURNING id",
            )
            .bind(TransactionId::new())
            .bind(household_id)
            .bind(account_id)
            .bind(row.values.booking_date)
            .bind(row.values.label.as_str())
            .bind(row.account_amount(account_kind).value())
            .bind(nature.as_str())
            .bind(TransactionSource::Import.as_str())
            .bind(statement_digest)
            .bind(line)
            .bind(file_name)
            .bind(row.values.signed_amount)
            .bind(row.source_record)
            .fetch_optional(&mut *transaction)
            .await?;

            let (status, transaction_id) = match inserted {
                Some(id) => (ImportStatus::Imported, id),
                None => {
                    let id = sqlx::query_scalar::<_, TransactionId>(
                        "SELECT id FROM transactions \
                         WHERE account_id = $1 AND csv_statement_digest = $2 AND csv_source_line = $3",
                    )
                    .bind(account_id)
                    .bind(statement_digest)
                    .bind(line)
                    .fetch_one(&mut *transaction)
                    .await?;
                    (ImportStatus::Duplicate, id)
                }
            };
            outcomes.push(ImportedRow {
                line,
                status,
                transaction_id,
            });
        }
        transaction.commit().await?;
        Ok(Some(outcomes))
    }
}
