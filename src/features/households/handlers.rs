use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{error::ApiError, types::HouseholdId};

use super::{
    HouseholdsModule,
    dto::{CreateHouseholdRequest, HouseholdResponse},
};

pub(super) async fn create(
    State(module): State<HouseholdsModule>,
    Json(request): Json<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<HouseholdResponse>), ApiError> {
    let household = module.service.create(request.into()).await?;
    Ok((StatusCode::CREATED, Json(household.into())))
}

pub(super) async fn get(
    State(module): State<HouseholdsModule>,
    Path(household_id): Path<HouseholdId>,
) -> Result<Json<HouseholdResponse>, ApiError> {
    let household = module.service.get(household_id).await?;
    Ok(Json(household.into()))
}
