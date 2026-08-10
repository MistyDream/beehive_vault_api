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
    error::{ApiError, required_text},
};

#[derive(Deserialize)]
pub struct CreateInstitutionRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct UpdateInstitutionRequest {
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionResponse {
    id: Uuid,
    household_id: Uuid,
    name: String,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub async fn create(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateInstitutionRequest>,
) -> Result<(StatusCode, Json<InstitutionResponse>), ApiError> {
    let name = required_text(request.name, "name")?;
    let institution = sqlx::query_as::<_, InstitutionResponse>(
        "INSERT INTO institutions (id, household_id, name) VALUES ($1, $2, $3) \
         RETURNING id, household_id, name, archived_at, created_at, updated_at",
    )
    .bind(Uuid::now_v7())
    .bind(household_id)
    .bind(name)
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(institution)))
}

pub async fn list(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<InstitutionResponse>>, ApiError> {
    let institutions = sqlx::query_as::<_, InstitutionResponse>(
        "SELECT id, household_id, name, archived_at, created_at, updated_at \
         FROM institutions WHERE household_id = $1 AND archived_at IS NULL ORDER BY lower(name)",
    )
    .bind(household_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(institutions))
}

pub async fn update(
    State(state): State<AppState>,
    Path((household_id, institution_id)): Path<(Uuid, Uuid)>,
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
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Institution not found".to_owned()))?;
    Ok(Json(institution))
}

pub async fn archive(
    State(state): State<AppState>,
    Path((household_id, institution_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE institutions SET archived_at = now(), updated_at = now() \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
    )
    .bind(household_id)
    .bind(institution_id)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Institution not found".to_owned()));
    }
    Ok(StatusCode::NO_CONTENT)
}
