use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{CategoryId, HouseholdId};

use super::{
    domain::{Category, CategoryKind},
    service::{CreateCategoryCommand, UpdateCategoryCommand},
};

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    name: String,
    kind: CategoryKind,
}

impl From<CreateCategoryRequest> for CreateCategoryCommand {
    fn from(request: CreateCategoryRequest) -> Self {
        Self {
            name: request.name,
            kind: request.kind,
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    name: String,
}

impl From<UpdateCategoryRequest> for UpdateCategoryCommand {
    fn from(request: UpdateCategoryRequest) -> Self {
        Self { name: request.name }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListQuery {
    pub kind: Option<CategoryKind>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    id: CategoryId,
    household_id: HouseholdId,
    name: String,
    kind: CategoryKind,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            household_id: category.household_id,
            name: category.name.into_string(),
            kind: category.kind,
            archived_at: category.archived_at,
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }
}
