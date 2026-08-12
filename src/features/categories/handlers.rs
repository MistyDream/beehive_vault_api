use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    error::ApiError,
    types::{CategoryId, HouseholdId},
};

use super::{
    CategoriesModule,
    dto::{CategoryListQuery, CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest},
};

pub(super) async fn create(
    State(module): State<CategoriesModule>,
    Path(household_id): Path<HouseholdId>,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    let category = module.service.create(household_id, request.into()).await?;
    Ok((StatusCode::CREATED, Json(category.into())))
}

pub(super) async fn list(
    State(module): State<CategoriesModule>,
    Path(household_id): Path<HouseholdId>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = module.service.list(household_id, query.kind).await?;
    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub(super) async fn update(
    State(module): State<CategoriesModule>,
    Path((household_id, category_id)): Path<(HouseholdId, CategoryId)>,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let category = module
        .service
        .update(household_id, category_id, request.into())
        .await?;
    Ok(Json(category.into()))
}

pub(super) async fn archive(
    State(module): State<CategoriesModule>,
    Path((household_id, category_id)): Path<(HouseholdId, CategoryId)>,
) -> Result<StatusCode, ApiError> {
    module.service.archive(household_id, category_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
