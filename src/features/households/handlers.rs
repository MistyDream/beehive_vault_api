use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, required_text},
    types::{CurrencyCode, HouseholdId},
};

use super::HouseholdsModule;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreateHouseholdRequest {
    name: String,
    base_currency: CurrencyCode,
    timezone: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(super) struct HouseholdResponse {
    id: HouseholdId,
    name: String,
    base_currency: CurrencyCode,
    timezone: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub(super) async fn create(
    State(module): State<HouseholdsModule>,
    Json(request): Json<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<HouseholdResponse>), ApiError> {
    let name = required_text(request.name, "name")?;
    let timezone = required_text(request.timezone, "timezone")?;

    let household = sqlx::query_as::<_, HouseholdResponse>(
        "INSERT INTO households (id, name, base_currency, timezone) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, name, base_currency, timezone, created_at, updated_at",
    )
    .bind(HouseholdId::new())
    .bind(name)
    .bind(request.base_currency)
    .bind(timezone)
    .fetch_one(module.database.pool())
    .await?;

    Ok((StatusCode::CREATED, Json(household)))
}

pub(super) async fn get(
    State(module): State<HouseholdsModule>,
    Path(household_id): Path<HouseholdId>,
) -> Result<Json<HouseholdResponse>, ApiError> {
    let household = sqlx::query_as::<_, HouseholdResponse>(
        "SELECT id, name, base_currency, timezone, created_at, updated_at \
         FROM households WHERE id = $1",
    )
    .bind(household_id)
    .fetch_optional(module.database.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))?;

    Ok(Json(household))
}
