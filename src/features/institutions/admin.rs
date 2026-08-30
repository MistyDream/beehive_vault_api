use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};

use crate::{database::Database, types::InstitutionId};

const MAX_INSTITUTION_NAME_LENGTH: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct Institution {
    pub id: InstitutionId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportReport {
    pub added: usize,
    pub unchanged: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstitutionCatalog {
    institutions: Vec<InstitutionCatalogEntry>,
}

impl InstitutionCatalog {
    pub fn from_json(value: &str) -> Result<Self, AdminError> {
        Ok(serde_json::from_str(value)?)
    }

    fn validated_names(self) -> Result<Vec<String>, AdminError> {
        let mut normalized_names = HashSet::new();
        let mut names = Vec::with_capacity(self.institutions.len());

        for entry in self.institutions {
            let name = institution_name(entry.name)?;
            let normalized_name = name.to_lowercase();
            if !normalized_names.insert(normalized_name) {
                return Err(AdminError::DuplicateCatalogName(name));
            }
            names.push(name);
        }

        Ok(names)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstitutionCatalogEntry {
    name: String,
}

#[derive(Clone)]
pub struct InstitutionAdminService {
    database: Database,
}

impl InstitutionAdminService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn list(&self) -> Result<Vec<Institution>, AdminError> {
        Ok(sqlx::query_as::<_, Institution>(
            "SELECT id, name FROM institutions ORDER BY lower(name), name, id",
        )
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn add(&self, name: String) -> Result<Institution, AdminError> {
        let name = institution_name(name)?;
        sqlx::query_as::<_, Institution>(
            "INSERT INTO institutions (id, name) VALUES ($1, $2) RETURNING id, name",
        )
        .bind(InstitutionId::new())
        .bind(name)
        .fetch_one(self.database.pool())
        .await
        .map_err(map_database_error)
    }

    pub async fn rename(
        &self,
        institution_id: InstitutionId,
        name: String,
    ) -> Result<Institution, AdminError> {
        let name = institution_name(name)?;
        sqlx::query_as::<_, Institution>(
            "UPDATE institutions SET name = $2, updated_at = now() WHERE id = $1 \
             RETURNING id, name",
        )
        .bind(institution_id)
        .bind(name)
        .fetch_optional(self.database.pool())
        .await
        .map_err(map_database_error)?
        .ok_or(AdminError::InstitutionNotFound(institution_id))
    }

    pub async fn import_catalog(
        &self,
        catalog: InstitutionCatalog,
    ) -> Result<ImportReport, AdminError> {
        let names = catalog.validated_names()?;
        let mut transaction = self.database.begin_transaction().await?;
        let mut added = 0;

        for name in &names {
            added += insert_if_missing(&mut transaction, name).await? as usize;
        }

        transaction.commit().await?;
        Ok(ImportReport {
            added,
            unchanged: names.len() - added,
        })
    }
}

async fn insert_if_missing(
    transaction: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<bool, AdminError> {
    let result =
        sqlx::query("INSERT INTO institutions (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(InstitutionId::new())
            .bind(name)
            .execute(&mut **transaction)
            .await?;
    Ok(result.rows_affected() == 1)
}

fn institution_name(value: String) -> Result<String, AdminError> {
    let name = value.trim();
    let length = name.chars().count();
    if length == 0 || length > MAX_INSTITUTION_NAME_LENGTH {
        return Err(AdminError::InvalidName);
    }
    Ok(name.to_owned())
}

fn map_database_error(error: sqlx::Error) -> AdminError {
    let constraint = error
        .as_database_error()
        .and_then(|database_error| database_error.constraint());
    if constraint == Some("institutions_name_unique") {
        AdminError::DuplicateInstitutionName
    } else {
        AdminError::Database(error)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("institution name must contain between 1 and 100 characters after trimming")]
    InvalidName,
    #[error("institution name already exists")]
    DuplicateInstitutionName,
    #[error("institution catalog contains the duplicate name '{0}'")]
    DuplicateCatalogName(String),
    #[error("institution '{0}' was not found")]
    InstitutionNotFound(InstitutionId),
    #[error("institution catalog is not valid JSON: {0}")]
    InvalidCatalog(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_rejects_duplicate_normalized_names() {
        let catalog = InstitutionCatalog::from_json(
            r#"{"institutions":[{"name":"Example Bank"},{"name":" example bank "}]}"#,
        )
        .unwrap();

        assert!(matches!(
            catalog.validated_names(),
            Err(AdminError::DuplicateCatalogName(_))
        ));
    }

    #[test]
    fn catalog_rejects_unknown_fields() {
        let result = InstitutionCatalog::from_json(
            r#"{"institutions":[{"name":"Example Bank","id":"unexpected"}]}"#,
        );

        assert!(matches!(result, Err(AdminError::InvalidCatalog(_))));
    }

    #[test]
    fn catalog_trims_valid_names_before_import() {
        let catalog =
            InstitutionCatalog::from_json(r#"{"institutions":[{"name":" Example Bank "}]}"#)
                .unwrap();

        assert_eq!(catalog.validated_names().unwrap(), vec!["Example Bank"]);
    }

    #[test]
    fn bundled_catalog_is_valid() {
        let catalog =
            InstitutionCatalog::from_json(include_str!("../../../catalog/institutions.json"))
                .unwrap();

        assert_eq!(catalog.validated_names().unwrap().len(), 15);
    }
}
