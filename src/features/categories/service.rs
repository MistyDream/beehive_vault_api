use crate::{
    error::{ApiError, ProblemKind},
    types::{CategoryId, HouseholdId},
};

use super::{
    domain::{Category, CategoryKind, CategoryName, NewCategory},
    repository::CategoryRepository,
};

pub struct CreateCategoryCommand {
    pub name: String,
    pub kind: CategoryKind,
}

pub struct UpdateCategoryCommand {
    pub name: String,
}

#[derive(Clone)]
pub struct CategoryService {
    repository: CategoryRepository,
}

impl CategoryService {
    pub fn new(repository: CategoryRepository) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        household_id: HouseholdId,
        command: CreateCategoryCommand,
    ) -> Result<Category, ApiError> {
        self.ensure_household_exists(household_id).await?;
        let name = category_name(command.name)?;
        Ok(self
            .repository
            .create(NewCategory {
                id: CategoryId::new(),
                household_id,
                name,
                kind: command.kind,
            })
            .await?)
    }

    pub async fn list(
        &self,
        household_id: HouseholdId,
        kind: Option<CategoryKind>,
    ) -> Result<Vec<Category>, ApiError> {
        self.ensure_household_exists(household_id).await?;
        Ok(self.repository.list(household_id, kind).await?)
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        category_id: CategoryId,
        command: UpdateCategoryCommand,
    ) -> Result<Category, ApiError> {
        let name = category_name(command.name)?;
        self.repository
            .update_name(household_id, category_id, &name)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::CategoryNotFound))
    }

    pub async fn archive(
        &self,
        household_id: HouseholdId,
        category_id: CategoryId,
    ) -> Result<(), ApiError> {
        if self.repository.archive(household_id, category_id).await? == 0 {
            return Err(ApiError::new(ProblemKind::CategoryNotFound));
        }
        Ok(())
    }

    async fn ensure_household_exists(&self, household_id: HouseholdId) -> Result<(), ApiError> {
        if !self.repository.household_exists(household_id).await? {
            return Err(ApiError::new(ProblemKind::HouseholdNotFound));
        }
        Ok(())
    }
}

fn category_name(value: String) -> Result<CategoryName, ApiError> {
    CategoryName::new(value)
        .map_err(|error| ApiError::body_validation("#/name", "invalid_length", error.to_string()))
}
