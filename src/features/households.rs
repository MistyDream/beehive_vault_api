use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, currency_code, required_text},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHouseholdRequest {
    name: String,
    base_currency: String,
    timezone: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdResponse {
    id: Uuid,
    name: String,
    base_currency: String,
    timezone: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<HouseholdResponse>), ApiError> {
    let name = required_text(request.name, "name")?;
    let base_currency = currency_code(request.base_currency)?;
    let timezone = required_text(request.timezone, "timezone")?;

    let household = sqlx::query_as::<_, HouseholdResponse>(
        "INSERT INTO households (id, name, base_currency, timezone) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, name, base_currency, timezone, created_at, updated_at",
    )
    .bind(Uuid::now_v7())
    .bind(name)
    .bind(base_currency)
    .bind(timezone)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(household)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> Result<Json<HouseholdResponse>, ApiError> {
    let household = sqlx::query_as::<_, HouseholdResponse>(
        "SELECT id, name, base_currency, timezone, created_at, updated_at \
         FROM households WHERE id = $1",
    )
    .bind(household_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))?;

    Ok(Json(household))
}
