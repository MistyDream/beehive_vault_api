use chrono::{DateTime, Utc};

use crate::{
    database::Database,
    features::categories::INITIAL_CATEGORIES,
    types::{CategoryId, CurrencyCode, HouseholdId, TimeZoneId},
};

use super::domain::{Household, NewHousehold};

#[derive(Clone)]
pub struct HouseholdRepository {
    database: Database,
}

impl HouseholdRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, household: NewHousehold) -> Result<Household, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let row = sqlx::query_as::<_, HouseholdRow>(
            "INSERT INTO households (id, name, base_currency, timezone) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, name, base_currency, timezone, created_at, updated_at",
        )
        .bind(household.id)
        .bind(household.name)
        .bind(household.base_currency)
        .bind(household.timezone.as_str())
        .fetch_one(&mut *transaction)
        .await?;

        for initial_category in INITIAL_CATEGORIES {
            sqlx::query(
                "INSERT INTO categories (id, household_id, name, kind) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(CategoryId::new())
            .bind(household.id)
            .bind(initial_category.name())
            .bind(initial_category.kind())
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Household::try_from(row)
    }

    pub async fn find(&self, household_id: HouseholdId) -> Result<Option<Household>, sqlx::Error> {
        let row = sqlx::query_as::<_, HouseholdRow>(
            "SELECT id, name, base_currency, timezone, created_at, updated_at \
             FROM households WHERE id = $1",
        )
        .bind(household_id)
        .fetch_optional(self.database.pool())
        .await?;
        row.map(Household::try_from).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct HouseholdRow {
    id: HouseholdId,
    name: String,
    base_currency: CurrencyCode,
    timezone: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<HouseholdRow> for Household {
    type Error = sqlx::Error;

    fn try_from(row: HouseholdRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            base_currency: row.base_currency,
            timezone: TimeZoneId::new(row.timezone)
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
