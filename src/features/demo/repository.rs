use crate::database::Database;

use super::DEMO_HOUSEHOLD_ID;

#[derive(Clone)]
pub struct DemoRepository {
    database: Database,
}

impl DemoRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn load(&self) -> Result<bool, DemoError> {
        let mut transaction = self.database.begin_transaction().await?;
        // Serialize loading and resetting the same fixed fixture identities.
        sqlx::query("SELECT pg_advisory_xact_lock(9000009)")
            .execute(&mut *transaction)
            .await?;
        let existing = demo_exists(&mut transaction).await?;
        if !existing {
            let conflict: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM institutions WHERE \
                 id = '019d9000-0000-7000-8000-00000000000b'::uuid AND name <> 'Demo Everyday Bank' \
                 OR id = '019d9000-0000-7000-8000-00000000000c'::uuid AND name <> 'Demo Investment Bank')",
            )
            .fetch_one(&mut *transaction)
            .await?;
            if conflict {
                return Err(DemoError::IdentityConflict);
            }
            sqlx::raw_sql(include_str!("../../../demo/load.sql"))
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(!existing)
    }

    pub async fn reset(&self) -> Result<bool, DemoError> {
        let mut transaction = self.database.begin_transaction().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(9000009)")
            .execute(&mut *transaction)
            .await?;
        let existing = demo_exists(&mut transaction).await?;
        if existing {
            // Remove transfer movements before transfers and category references.
            sqlx::query("DELETE FROM transactions WHERE household_id = $1::uuid")
                .bind(DEMO_HOUSEHOLD_ID)
                .execute(&mut *transaction)
                .await?;
            sqlx::query("DELETE FROM households WHERE id = $1::uuid")
                .bind(DEMO_HOUSEHOLD_ID)
                .execute(&mut *transaction)
                .await?;
            // Lock the global institutions before checking references from other households.
            sqlx::query(
                "SELECT id FROM institutions WHERE id IN \
                 ('019d9000-0000-7000-8000-00000000000b'::uuid, \
                  '019d9000-0000-7000-8000-00000000000c'::uuid) FOR UPDATE",
            )
            .execute(&mut *transaction)
            .await?;
            // The two global institutions can have been reused by another household.
            sqlx::query(
                "DELETE FROM institutions WHERE \
                 (id = '019d9000-0000-7000-8000-00000000000b'::uuid AND name = 'Demo Everyday Bank' \
                  OR id = '019d9000-0000-7000-8000-00000000000c'::uuid AND name = 'Demo Investment Bank') \
                 AND NOT EXISTS (SELECT 1 FROM accounts WHERE institution_id = institutions.id)",
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(existing)
    }
}

async fn demo_exists(connection: &mut sqlx::PgConnection) -> Result<bool, DemoError> {
    let identity = sqlx::query_as::<_, (String, String, String)>(
        "SELECT name, base_currency, timezone FROM households WHERE id = $1::uuid FOR UPDATE",
    )
    .bind(DEMO_HOUSEHOLD_ID)
    .fetch_optional(connection)
    .await?;
    match identity {
        None => Ok(false),
        Some((name, currency, timezone))
            if name == "Demo Household" && currency == "EUR" && timezone == "Europe/Paris" =>
        {
            Ok(true)
        }
        Some(_) => Err(DemoError::IdentityConflict),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("a reserved demo identity has different attributes; no fixture was changed")]
    IdentityConflict,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
