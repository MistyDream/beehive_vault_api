use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, required_text},
    types::{HouseholdId, InstitutionId},
};

use super::InstitutionsModule;

#[derive(Deserialize)]
pub(super) struct CreateInstitutionRequest {
    name: String,
}

#[derive(Deserialize)]
pub(super) struct UpdateInstitutionRequest {
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstitutionResponse {
    id: InstitutionId,
    household_id: HouseholdId,
    name: String,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub(super) async fn create(
    State(module): State<InstitutionsModule>,
    Path(household_id): Path<HouseholdId>,
    Json(request): Json<CreateInstitutionRequest>,
) -> Result<(StatusCode, Json<InstitutionResponse>), ApiError> {
    let name = required_text(request.name, "name")?;
    let institution = sqlx::query_as::<_, InstitutionResponse>(
        "INSERT INTO institutions (id, household_id, name) VALUES ($1, $2, $3) \
         RETURNING id, household_id, name, archived_at, created_at, updated_at",
    )
    .bind(InstitutionId::new())
    .bind(household_id)
    .bind(name)
    .fetch_one(module.database.pool())
    .await?;
    Ok((StatusCode::CREATED, Json(institution)))
}

pub(super) async fn list(
    State(module): State<InstitutionsModule>,
    Path(household_id): Path<HouseholdId>,
) -> Result<Json<Vec<InstitutionResponse>>, ApiError> {
    let institutions = sqlx::query_as::<_, InstitutionResponse>(
        "SELECT id, household_id, name, archived_at, created_at, updated_at \
         FROM institutions WHERE household_id = $1 AND archived_at IS NULL ORDER BY lower(name)",
    )
    .bind(household_id)
    .fetch_all(module.database.pool())
    .await?;
    Ok(Json(institutions))
}

pub(super) async fn update(
    State(module): State<InstitutionsModule>,
    Path((household_id, institution_id)): Path<(HouseholdId, InstitutionId)>,
    Json(request): Json<UpdateInstitutionRequest>,
) -> Result<Json<InstitutionResponse>, ApiError> {
    let name = required_text(request.name, "name")?;
    let institution = sqlx::query_as::<_, InstitutionResponse>(
        "UPDATE institutions SET name = $3, updated_at = now() \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL \
         RETURNING id, household_id, name, archived_at, created_at, updated_at",
    )
    .bind(household_id)
    .bind(institution_id)
    .bind(name)
    .fetch_optional(module.database.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Institution not found".to_owned()))?;
    Ok(Json(institution))
}

pub(super) async fn archive(
    State(module): State<InstitutionsModule>,
    Path((household_id, institution_id)): Path<(HouseholdId, InstitutionId)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE institutions SET archived_at = now(), updated_at = now() \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
    )
    .bind(household_id)
    .bind(institution_id)
    .execute(module.database.pool())
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Institution not found".to_owned()));
    }
    Ok(StatusCode::NO_CONTENT)
}
