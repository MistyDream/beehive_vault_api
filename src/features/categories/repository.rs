use chrono::{DateTime, Utc};

use crate::{
    database::Database,
    types::{CategoryId, HouseholdId},
};

use super::domain::{Category, CategoryKind, CategoryName, NewCategory};

#[derive(Clone)]
pub struct CategoryRepository {
    database: Database,
}

impl CategoryRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn household_exists(&self, household_id: HouseholdId) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM households WHERE id = $1)")
            .bind(household_id)
            .fetch_one(self.database.pool())
            .await
    }

    pub async fn create(&self, category: NewCategory) -> Result<Category, sqlx::Error> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "INSERT INTO categories (id, household_id, name, kind) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, household_id, name, kind, archived_at, created_at, updated_at",
        )
        .bind(category.id)
        .bind(category.household_id)
        .bind(category.name.as_str())
        .bind(category.kind.as_str())
        .fetch_one(self.database.pool())
        .await?;
        Category::try_from(row)
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        kind: Option<CategoryKind>,
    ) -> Result<Vec<Category>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            "SELECT id, household_id, name, kind, archived_at, created_at, updated_at \
             FROM categories \
             WHERE household_id = $1 AND archived_at IS NULL \
               AND ($2::text IS NULL OR kind = $2) \
             ORDER BY kind, lower(name), id",
        )
        .bind(household_id)
        .bind(kind.map(CategoryKind::as_str))
        .fetch_all(self.database.pool())
        .await?;
        rows.into_iter().map(Category::try_from).collect()
    }

    pub async fn update_name(
        &self,
        household_id: HouseholdId,
        category_id: CategoryId,
        name: &CategoryName,
    ) -> Result<Option<Category>, sqlx::Error> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "UPDATE categories SET name = $3, updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND archived_at IS NULL \
             RETURNING id, household_id, name, kind, archived_at, created_at, updated_at",
        )
        .bind(household_id)
        .bind(category_id)
        .bind(name.as_str())
        .fetch_optional(self.database.pool())
        .await?;
        row.map(Category::try_from).transpose()
    }

    pub async fn archive(
        &self,
        household_id: HouseholdId,
        category_id: CategoryId,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE categories SET archived_at = now(), updated_at = now() \
             WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
        )
        .bind(household_id)
        .bind(category_id)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: CategoryId,
    household_id: HouseholdId,
    name: String,
    kind: String,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<CategoryRow> for Category {
    type Error = sqlx::Error;

    fn try_from(row: CategoryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            household_id: row.household_id,
            name: CategoryName::new(row.name)
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            kind: CategoryKind::try_from(row.kind.as_str())
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
