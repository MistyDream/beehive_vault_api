use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath},
    types::HouseholdId,
};

use super::{
    HouseholdsModule,
    dto::{CreateHouseholdRequest, HouseholdResponse},
};

pub(super) async fn create(
    State(module): State<HouseholdsModule>,
    ApiJson(request): ApiJson<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<HouseholdResponse>), ApiError> {
    let household = module.service.create(request.into()).await?;
    Ok((StatusCode::CREATED, Json(household.into())))
}

pub(super) async fn list(
    State(module): State<HouseholdsModule>,
) -> Result<Json<Vec<HouseholdResponse>>, ApiError> {
    let households = module.service.list().await?;
    Ok(Json(households.into_iter().map(Into::into).collect()))
}

pub(super) async fn get(
    State(module): State<HouseholdsModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
) -> Result<Json<HouseholdResponse>, ApiError> {
    let household = module.service.get(household_id).await?;
    Ok(Json(household.into()))
}
