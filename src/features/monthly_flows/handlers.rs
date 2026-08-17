use std::str::FromStr;

use axum::{
    Json,
    extract::{Path, State},
};

use crate::{error::ApiError, types::HouseholdId};

use super::{MonthlyFlowsModule, domain::Month, dto::MonthlyFlowResponse};

pub(super) async fn get(
    State(module): State<MonthlyFlowsModule>,
    Path((household_id, month)): Path<(HouseholdId, String)>,
) -> Result<Json<MonthlyFlowResponse>, ApiError> {
    let month = Month::from_str(&month).map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let report = module.service.get(household_id, month).await?;

    Ok(Json(report.into()))
}
