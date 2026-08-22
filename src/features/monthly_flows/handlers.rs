use std::str::FromStr;

use axum::{Json, extract::State};

use crate::{
    error::{ApiError, InvalidParameter, InvalidParameterLocation},
    extract::ApiPath,
    types::HouseholdId,
};

use super::{MonthlyFlowsModule, domain::Month, dto::MonthlyFlowResponse};

pub(super) async fn get(
    State(module): State<MonthlyFlowsModule>,
    ApiPath((household_id, month)): ApiPath<(HouseholdId, String)>,
) -> Result<Json<MonthlyFlowResponse>, ApiError> {
    let month = Month::from_str(&month).map_err(|error| {
        ApiError::validation(InvalidParameter::new(
            InvalidParameterLocation::Path,
            "#/month",
            "invalid_month",
            error.to_string(),
        ))
    })?;
    let report = module.service.get(household_id, month).await?;

    Ok(Json(report.into()))
}
