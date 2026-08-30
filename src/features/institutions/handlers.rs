use axum::{Json, extract::State};
use serde::Serialize;

use crate::{error::ApiError, types::InstitutionId};

use super::InstitutionsModule;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct InstitutionResponse {
    id: InstitutionId,
    name: String,
}

pub(super) async fn list(
    State(module): State<InstitutionsModule>,
) -> Result<Json<Vec<InstitutionResponse>>, ApiError> {
    let institutions = sqlx::query_as::<_, InstitutionResponse>(
        "SELECT id, name FROM institutions ORDER BY lower(name), name, id",
    )
    .fetch_all(module.database.pool())
    .await?;

    Ok(Json(institutions))
}
